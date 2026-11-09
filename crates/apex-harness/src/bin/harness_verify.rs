use std::path::PathBuf;
use clap::Parser;
use apex_harness::{FeatureRegistry, FeatureState, ThreeLayerGate};

#[derive(Parser, Debug)]
#[command(author, version, about = "Apex Harness Verification Gatekeeper")]
struct Args {
    /// Specific feature ID to verify (e.g. F01)
    #[arg(short, long)]
    feature: Option<String>,

    /// Verify all features sequentially
    #[arg(short, long)]
    all: bool,

    /// Path to features.json
    #[arg(short, long, default_value = "features.json")]
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    println!("🛡️  Apex Harness Verification Gatekeeper Initiated");
    println!("    State Primitive File: {}", args.path.display());

    let mut registry = FeatureRegistry::load_from_file(&args.path)?;
    let mut verified_count = 0;
    let mut failed_count = 0;

    if let Some(target_id) = args.feature {
        let feature = registry
            .get_feature_mut(&target_id)
            .ok_or_else(|| anyhow::anyhow!("Feature {target_id} not found in registry"))?;

        println!("\n🔍 Auditing Feature [{}]: {}", feature.id, feature.behavior);
        println!("   Command: {}", feature.verification);

        let outcome = ThreeLayerGate::certify_feature(feature)?;
        if outcome.success {
            println!("   ✅ PASS: Feature transitioned to {:?}", FeatureState::Passing);
            println!("   Evidence: {}", feature.evidence.as_deref().unwrap_or(""));
            verified_count += 1;
        } else {
            println!("   ❌ FAIL: Verification command failed with exit code {:?}", outcome.exit_code);
            eprintln!("   Stderr: {}", outcome.stderr);
            failed_count += 1;
        }
    } else if args.all {
        for feature in &mut registry.features {
            println!("\n🔍 Auditing Feature [{}]: {}", feature.id, feature.behavior);
            println!("   Command: {}", feature.verification);

            let outcome = ThreeLayerGate::certify_feature(feature)?;
            if outcome.success {
                println!("   ✅ PASS: Feature transitioned to {:?}", FeatureState::Passing);
                verified_count += 1;
            } else {
                println!("   ❌ FAIL: Exit code {:?}", outcome.exit_code);
                failed_count += 1;
            }
        }
    } else {
        println!("Please provide either --feature <ID> or --all");
        return Ok(());
    }

    registry.save_to_file(&args.path)?;
    println!("\n─────────────────────────────────────────────────────────────");
    println!("📊 Summary: {verified_count} Certified Passing | {failed_count} Blocked/Failed");
    println!("   Back-pressure remaining: {} features", registry.back_pressure_count());

    if failed_count > 0 {
        std::process::exit(1);
    }

    Ok(())
}
