/**
 * Callback when the user submits the form
 *
 * @param event {Event} the event object
 */
const handleInputSubmissionStart = (event) => {
  const input = event.target;

  // Disable the input
  input.setAttribute("disabled", "");
};

/**
 * Callback when the user finishes submitting the form
 *
 * @param event {Event} the event object
 */
const handleInputSubmissionFinished = (event) => {
  const input = event.target;

  // Re-enable the input
  input.removeAttribute("disabled");

  // Clear its value
  input.value = "";

  // Focus it for quick typing again
  input.focus();
};
