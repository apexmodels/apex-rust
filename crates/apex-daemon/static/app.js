/**
 * ApexPhi-Think-14B Voice, Web & Advanced Reasoning Frontend Controller
 */

let messageHistory = [];
let isGenerating = false;
let currentThinkingDepth = "standard";
let recognition = null;
let isRecording = false;
let currentUtterance = null;

// DOM Elements
const chatContainer = document.getElementById("chatContainer");
const messagesList = document.getElementById("messagesList");
const welcomeBanner = document.getElementById("welcomeBanner");
const promptInput = document.getElementById("promptInput");
const btnSend = document.getElementById("btnSend");
const btnVoiceInput = document.getElementById("btnVoiceInput");
const webSearchToggle = document.getElementById("webSearchToggle");
const autoSpeakToggle = document.getElementById("autoSpeakToggle");
const voiceSelect = document.getElementById("voiceSelect");
const btnStopVoice = document.getElementById("btnStopVoice");
const statusIndicator = document.getElementById("statusIndicator");
const cognitiveStatusBar = document.getElementById("cognitiveStatusBar");
const cognitiveStatusText = document.getElementById("cognitiveStatusText");
const activeMetrics = document.getElementById("activeMetrics");

// Initialize on Load
document.addEventListener("DOMContentLoaded", () => {
    initThinkingDepthSelector();
    initSpeechRecognition();
    populateVoices();
    checkHealth();
    setupInputListeners();
});

// 1. Thinking Depth Selector
function initThinkingDepthSelector() {
    const depthButtons = document.querySelectorAll("#thinkDepthControl .segment-btn");
    depthButtons.forEach(btn => {
        btn.addEventListener("click", () => {
            depthButtons.forEach(b => b.classList.remove("active"));
            btn.classList.add("active");
            currentThinkingDepth = btn.getAttribute("data-depth");
            console.log("Active thinking depth:", currentThinkingDepth);
        });
    });
}

// 2. Health & Model Status
async function checkHealth() {
    try {
        const resp = await fetch("/api/health");
        if (resp.ok) {
            const data = await resp.json();
            if (data.model_ready) {
                statusIndicator.className = "status-badge status-online";
                statusIndicator.textContent = "● Ready (Phi-4 Think 14B)";
            } else if (data.ollama_connected) {
                statusIndicator.className = "status-badge status-loading";
                statusIndicator.textContent = "● Ollama Online (Pulling Model...)";
            } else {
                statusIndicator.className = "status-badge status-loading";
                statusIndicator.textContent = "● Ollama Offline";
            }
        }
    } catch (e) {
        statusIndicator.className = "status-badge status-loading";
        statusIndicator.textContent = "● Backend Disconnected";
    }
}

// 3. Populate Audio Voices
function populateVoices() {
    if ("speechSynthesis" in window) {
        const loadBrowserVoices = () => {
            const voices = window.speechSynthesis.getVoices();
            if (voices.length > 0) {
                voiceSelect.innerHTML = "";
                voices.forEach(v => {
                    if (v.lang.startsWith("en")) {
                        const opt = document.createElement("option");
                        opt.value = v.name;
                        opt.textContent = `${v.name} (${v.lang})`;
                        if (v.name.includes("Samantha") || v.name.includes("Natural") || v.default) {
                            opt.selected = true;
                        }
                        voiceSelect.appendChild(opt);
                    }
                });
            }
        };
        window.speechSynthesis.onvoiceschanged = loadBrowserVoices;
        loadBrowserVoices();
    }
}

// 4. Speech Recognition (Voice Input / STT)
function initSpeechRecognition() {
    const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
    if (!SpeechRecognition) {
        btnVoiceInput.title = "Speech Recognition not supported in this browser. Use Chrome/Safari/Edge.";
        btnVoiceInput.style.opacity = "0.5";
        return;
    }

    recognition = new SpeechRecognition();
    recognition.continuous = false;
    recognition.interimResults = true;
    recognition.lang = "en-US";

    recognition.onstart = () => {
        isRecording = true;
        btnVoiceInput.classList.add("recording");
        promptInput.placeholder = "Listening... speak now...";
    };

    recognition.onresult = (event) => {
        let transcript = "";
        for (let i = event.resultIndex; i < event.results.length; i++) {
            transcript += event.results[i][0].transcript;
        }
        promptInput.value = transcript;
        autoResizeInput();
    };

    recognition.onerror = (event) => {
        console.warn("Speech recognition error:", event.error);
        stopVoiceRecording();
    };

    recognition.onend = () => {
        stopVoiceRecording();
    };
}

function toggleVoiceRecording() {
    if (!recognition) {
        alert("Speech Recognition is not supported by your current browser. Please use Chrome, Safari, or an Electron webview.");
        return;
    }
    if (isRecording) {
        recognition.stop();
        stopVoiceRecording();
    } else {
        try {
            recognition.start();
        } catch (e) {
            console.error("Could not start recognition:", e);
        }
    }
}

function stopVoiceRecording() {
    isRecording = false;
    btnVoiceInput.classList.remove("recording");
    promptInput.placeholder = "Ask a mathematical theorem, code design, or web inquiry (or click 🎙️ to speak)...";
}

// 5. Speech Synthesis (Voice Output / TTS)
function speakText(rawText) {
    if (!rawText) return;

    // Filter out <think> blocks and code blocks for spoken audio
    let textToSpeak = rawText;
    if (textToSpeak.includes("</think>")) {
        textToSpeak = textToSpeak.split("</think>")[1];
    } else {
        textToSpeak = textToSpeak.replace(/<think>[\s\S]*?<\/think>/g, "");
    }

    textToSpeak = textToSpeak
        .replace(/```[\s\S]*?```/g, " Code block omitted. ")
        .replace(/`([^`]+)`/g, "$1")
        .replace(/\[([^\]]+)\]\([^\)]+\)/g, "$1")
        .replace(/[#*_~]/g, "")
        .trim();

    if (!textToSpeak) return;

    // Stop existing speech
    stopCurrentSpeech();

    if ("speechSynthesis" in window) {
        currentUtterance = new SpeechSynthesisUtterance(textToSpeak);
        const selectedVoiceName = voiceSelect.value;
        const voices = window.speechSynthesis.getVoices();
        const found = voices.find(v => v.name === selectedVoiceName);
        if (found) currentUtterance.voice = found;
        currentUtterance.rate = 1.05;

        currentUtterance.onstart = () => {
            btnStopVoice.classList.remove("hidden");
        };
        currentUtterance.onend = () => {
            btnStopVoice.classList.add("hidden");
        };
        currentUtterance.onerror = () => {
            btnStopVoice.classList.add("hidden");
        };

        window.speechSynthesis.speak(currentUtterance);
    } else {
        // Fallback to backend macOS `say` endpoint
        fetch("/api/voice/speak", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ text: textToSpeak, voice: voiceSelect.value })
        }).then(() => {
            btnStopVoice.classList.remove("hidden");
        });
    }
}

function stopCurrentSpeech() {
    if ("speechSynthesis" in window) {
        window.speechSynthesis.cancel();
    }
    fetch("/api/voice/stop", { method: "POST" });
    btnStopVoice.classList.add("hidden");
}

// 6. Send & Stream Generation
function setupInputListeners() {
    promptInput.addEventListener("keydown", (e) => {
        if (e.key === "Enter" && !e.shiftKey) {
            e.preventDefault();
            handleSendClick();
        }
    });

    promptInput.addEventListener("input", autoResizeInput);
}

function autoResizeInput() {
    promptInput.style.height = "auto";
    promptInput.style.height = Math.min(promptInput.scrollHeight, 180) + "px";
}

function sendQuickPrompt(promptText) {
    promptInput.value = promptText;
    autoResizeInput();
    handleSendClick();
}

async function handleSendClick() {
    const text = promptInput.value.trim();
    if (!text || isGenerating) return;

    if (welcomeBanner) {
        welcomeBanner.classList.add("hidden");
    }

    // Append User Message
    appendMessage("user", text);
    promptInput.value = "";
    promptInput.style.height = "auto";
    isGenerating = true;
    btnSend.disabled = true;

    messageHistory.push({ role: "user", content: text });

    // Prepare Assistant Card with Thinking Accordion + Answer Area
    const assistantCard = createAssistantCard();
    messagesList.appendChild(assistantCard.card);
    scrollToBottom();

    // Show cognitive status bar
    cognitiveStatusBar.classList.remove("hidden");
    cognitiveStatusText.textContent = "Formulating invariant & web exploration...";

    const t0 = performance.now();
    let thinkingText = "";
    let answerText = "";
    let spokenSummary = "";
    let timerInterval = null;

    // Live thinking timer
    timerInterval = setInterval(() => {
        const elapsed = ((performance.now() - t0) / 1000).toFixed(1);
        if (assistantCard.thinkingTitle) {
            assistantCard.thinkingTitle.textContent = `Reasoning in progress (${elapsed}s)...`;
        }
    }, 200);

    try {
        const response = await fetch("/api/chat/stream", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
                messages: messageHistory,
                web_search: webSearchToggle.checked,
                think_depth: currentThinkingDepth
            })
        });

        if (!response.ok) {
            throw new Error(`Server returned HTTP ${response.status}`);
        }

        const reader = response.body.getReader();
        const decoder = new TextDecoder();
        let buffer = "";

        while (true) {
            const { value, done } = await reader.read();
            if (done) break;

            buffer += decoder.decode(value, { stream: true });
            const events = buffer.split("\n\n");
            buffer = events.pop(); // keep last incomplete chunk

            for (const ev of events) {
                if (!ev.trim()) continue;
                const lines = ev.split("\n");
                let eventType = "";
                let dataStr = "";

                for (const line of lines) {
                    if (line.startsWith("event: ")) {
                        eventType = line.slice(7).trim();
                    } else if (line.startsWith("data: ")) {
                        dataStr = line.slice(6).trim();
                    }
                }

                if (!dataStr) continue;
                let payload = {};
                try {
                    payload = JSON.parse(dataStr);
                } catch (e) {
                    continue;
                }

                if (eventType === "status") {
                    cognitiveStatusText.textContent = payload.message || "Reasoning...";
                } else if (eventType === "sources") {
                    renderSources(assistantCard.sourcesBox, payload.sources);
                } else if (eventType === "think_token") {
                    thinkingText += payload.token;
                    assistantCard.thinkingBody.textContent = thinkingText;
                    scrollToBottom();
                } else if (eventType === "think_end") {
                    clearInterval(timerInterval);
                    const elapsed = payload.elapsed || ((performance.now() - t0) / 1000).toFixed(1);
                    assistantCard.thinkingTitle.textContent = `Completed Reasoning Process (${elapsed}s)`;
                    assistantCard.thinkingPulse.style.display = "none";
                    cognitiveStatusText.textContent = "Synthesizing verified solution...";
                } else if (eventType === "answer_token") {
                    answerText += payload.token;
                    assistantCard.answerBubble.innerHTML = marked.parse(answerText);
                    assistantCard.answerBubble.querySelectorAll("pre code").forEach((el) => {
                        hljs.highlightElement(el);
                    });
                    scrollToBottom();
                } else if (eventType === "done") {
                    clearInterval(timerInterval);
                    cognitiveStatusBar.classList.add("hidden");
                    spokenSummary = payload.spoken_summary || answerText;
                    activeMetrics.textContent = `⚡ Latency: ${payload.total_time}s | Tokens: ${payload.tokens} | ${payload.speed} tok/s`;

                    messageHistory.push({ role: "assistant", content: (thinkingText ? `<think>${thinkingText}</think>` : "") + answerText });

                    // Auto Speak if enabled
                    if (autoSpeakToggle.checked) {
                        speakText(spokenSummary || answerText);
                    }
                } else if (eventType === "error") {
                    assistantCard.answerBubble.innerHTML = `<span style="color: #f87171;">Error: ${payload.error}</span>`;
                }
            }
        }
    } catch (err) {
        clearInterval(timerInterval);
        cognitiveStatusBar.classList.add("hidden");
        assistantCard.answerBubble.innerHTML = `<span style="color: #f87171;">Connection Error: ${err.message}</span>`;
    } finally {
        clearInterval(timerInterval);
        isGenerating = false;
        btnSend.disabled = false;
        cognitiveStatusBar.classList.add("hidden");
    }
}

// Helper: Append User Message
function appendMessage(role, text) {
    const card = document.createElement("div");
    card.className = `message-card ${role}`;

    const bubble = document.createElement("div");
    bubble.className = "bubble";
    bubble.textContent = text;

    card.appendChild(bubble);
    messagesList.appendChild(card);
}

// Helper: Create Assistant Card Template
function createAssistantCard() {
    const card = document.createElement("div");
    card.className = "message-card assistant";

    // Sources Box (Hidden until sources arrive)
    const sourcesBox = document.createElement("div");
    sourcesBox.className = "sources-box hidden";
    card.appendChild(sourcesBox);

    // Thinking Accordion
    const accordion = document.createElement("div");
    accordion.className = "thinking-accordion";

    const header = document.createElement("div");
    header.className = "thinking-header";

    const titleDiv = document.createElement("div");
    titleDiv.className = "thinking-title";

    const pulse = document.createElement("span");
    pulse.className = "thinking-pulse";

    const titleSpan = document.createElement("span");
    titleSpan.textContent = "Reasoning in progress...";

    titleDiv.appendChild(pulse);
    titleDiv.appendChild(titleSpan);

    const chevron = document.createElement("span");
    chevron.className = "thinking-chevron";
    chevron.textContent = "▼";

    header.appendChild(titleDiv);
    header.appendChild(chevron);

    const body = document.createElement("div");
    body.className = "thinking-body";

    header.addEventListener("click", () => {
        accordion.classList.toggle("collapsed");
    });

    accordion.appendChild(header);
    accordion.appendChild(body);
    card.appendChild(accordion);

    // Answer Bubble
    const answerBubble = document.createElement("div");
    answerBubble.className = "answer-bubble";
    answerBubble.innerHTML = "<span style='color: #94a3b8;'>Formulating response...</span>";
    card.appendChild(answerBubble);

    // Message Actions Bar
    const actionsBar = document.createElement("div");
    actionsBar.className = "message-actions";

    const btnSpeak = document.createElement("button");
    btnSpeak.className = "action-btn";
    btnSpeak.innerHTML = "🔊 Listen";
    btnSpeak.onclick = () => {
        const fullContent = answerBubble.innerText;
        speakText(fullContent);
    };

    const btnCopy = document.createElement("button");
    btnCopy.className = "action-btn";
    btnCopy.innerHTML = "📋 Copy";
    btnCopy.onclick = () => {
        navigator.clipboard.writeText(answerBubble.innerText);
        btnCopy.innerHTML = "✓ Copied!";
        setTimeout(() => btnCopy.innerHTML = "📋 Copy", 2000);
    };

    actionsBar.appendChild(btnSpeak);
    actionsBar.appendChild(btnCopy);
    card.appendChild(actionsBar);

    return {
        card,
        sourcesBox,
        thinkingAccordion: accordion,
        thinkingTitle: titleSpan,
        thinkingPulse: pulse,
        thinkingBody: body,
        answerBubble
    };
}

// Render Web Sources
function renderSources(container, sources) {
    if (!sources || sources.length === 0) return;
    container.innerHTML = "";
    container.classList.remove("hidden");

    const header = document.createElement("div");
    header.className = "sources-header";
    header.innerHTML = "🌐 Grounded Web Sources";
    container.appendChild(header);

    const pills = document.createElement("div");
    pills.className = "sources-pills";

    sources.forEach(src => {
        const pill = document.createElement("a");
        pill.className = "source-pill";
        pill.href = src.url;
        pill.target = "_blank";
        pill.rel = "noopener noreferrer";
        pill.title = src.snippet;
        pill.innerHTML = `<span>🔗</span> <strong>[Source ${src.id}]</strong> ${src.title.slice(0, 35)}...`;
        pills.appendChild(pill);
    });

    container.appendChild(pills);
}

function scrollToBottom() {
    chatContainer.scrollTop = chatContainer.scrollHeight;
}
