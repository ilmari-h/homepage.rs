window.onload = function () {
  const tagLinks = document.getElementById("post-tags").querySelectorAll("a");
  const urlParams = new Proxy(new URLSearchParams(window.location.search), {
    get: (searchParams, prop) => searchParams.get(prop),
  });
  if (!urlParams.tags) {
    return;
  }
  const tags = urlParams.tags.split(" ");

  // Append existing tags to links.
  for (const link of tagLinks) {
    const originalTag = link.textContent.trim();
    const tagsNoDupl = tags.filter((t) => t != originalTag);
    if (tagsNoDupl.length) {
      const found = link.href.indexOf("?tags=");
      if (found == -1) {
        link.href += "?tags=";
      } else {
        link.href += "+";
      }
      link.href += tagsNoDupl.join("+");
    }
  }
};
