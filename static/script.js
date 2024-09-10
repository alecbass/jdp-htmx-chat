/**
 * Callback when the user submits the form
 *
 * @param event {Event} the event object
 */
const handleInputSubmissionStart = (event) => {
  /** @type {HTMLInputElement | null} */
  const input = event.target;

  if (input) {
    // Disable the input
    input.setAttribute("disabled", "");
  }
};

/**
 * Callback when the user finishes submitting the form
 *
 * @param event {Event} the event object
 */
const handleInputSubmissionFinished = (event) => {
  /** @type {HTMLInputElement | null} */
  const input = event.target;

  if (!input) {
    return;
  }

  // Re-enable the input
  input.removeAttribute("disabled");

  // Clear its value
  input.value = "";

  // Focus it for quick typing again
  input.focus();
};

setTimeout(() => {
  const ws = new WebSocket("wss://jdp-chat-room.onrender.com");
  // const ws = new WebSocket("ws://0.0.0.0:8001");

  ws.addEventListener("open", (event) => {
    ws.send("WS opened");
  });

  ws.addEventListener("message", (event) => {
    console.debug(`Messagge: ${event.data}`);
  });

  ws.addEventListener("error", (event) => {
    console.debug("WS error:", event);
  });

  ws.addEventListener("close", () => {
    console.debug("Websocket closed");
  });
}, 0);
