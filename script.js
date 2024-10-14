function applyEvents() {
  for (const sound of ["open", "mmm", "uhh", "yes"]) {
    for (const element of document.querySelectorAll(`.sound-${sound}`)) {
      if (element.dataset[`${sound}SoundApplied`]) continue;
      element.addEventListener("click", () =>
        new Audio(`/assets/${sound}.mp3`).play(),
      );
      element.dataset[`${sound}SoundApplied`] = "true";
    }
  }
}
const observer = new MutationObserver(applyEvents);
observer.observe(document.body, {
  attributes: false,
  childList: true,
  subtree: true,
});
applyEvents();
