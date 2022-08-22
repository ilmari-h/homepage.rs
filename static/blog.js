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
  const parent = content.querySelector(".blog-post-content");
  const preElements = content.querySelectorAll("pre");

  let id = 0;
  for (let pre of preElements) {
    const btnNode = document.createElement("button");

    // Wrap pre in a div to position the copy button correctly.
    const wrapperDiv = document.createElement("div");
    wrapperDiv.classList.add("pre-wrapper");
    pre.id = `pre-${id}`;
    id++;
    parent.replaceChild(wrapperDiv, pre);
    wrapperDiv.appendChild(pre);

    btnNode.classList.add("pre-copy-button");
    btnNode.onclick = () => {
      let codeBlock = btnNode.parentNode.textContent;
      if (!navigator.clipboard) {
        fallbackCopyTextToClipboard(text);
        return;
      }
      navigator.clipboard.writeText(codeBlock).then(
        function () {
          btnNode.classList.add("success");
          setTimeout(function () {
            btnNode.classList.remove("success");
          }, 1000);
        },
        function (err) {
          console.error("Could not copy text.", err);
        }
      );
    };

    pre.appendChild(btnNode);
  }
};
