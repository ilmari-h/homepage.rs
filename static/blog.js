function fallbackCopyTextToClipboard(text) {
  var textArea = document.createElement("textarea");
  textArea.value = text;

  // Avoid scrolling to bottom
  textArea.style.top = "0";
  textArea.style.left = "0";
  textArea.style.position = "fixed";

  document.body.appendChild(textArea);
  textArea.focus();
  textArea.select();

  try {
    let result = document.execCommand("copy");
    if (result != "successful") {
      throw result;
    }
  } catch (err) {
    console.error("Fallback: Could not copy text", err);
  }
  document.body.removeChild(textArea);
}

window.onload = function () {
  // Add copy-to-clipboard option for code tags
  const content = document.getElementById("content");
  const preElements = content.querySelectorAll("pre");

  for (let pre of preElements) {
    const btnNode = document.createElement("button");
    btnNode.classList.add("pre-copy-button");
    btnNode.onclick = () => {
      let codeBlock = btnNode.parentNode.textContent;
      if (!navigator.clipboard) {
        fallbackCopyTextToClipboard(text);
        return;
      }
      navigator.clipboard.writeText(codeBlock).then(
        function () {},
        function (err) {
          console.error("Could not copy text.", err);
        }
      );
    };
    console.log(btnNode);

    pre.appendChild(btnNode);
  }
};
