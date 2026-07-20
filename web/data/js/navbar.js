// Mainly for setting .navbar_item > a
// to be width: fit-content. If it won't overflow. Else
// min-content if does. So it turned into single icon
// instead ugly widen one

const STATE = new WeakMap()

function getNaturalSize(node) {
  // Clone the node to avoid altering the actual DOM
  const clone = node.cloneNode(true);
  
  // Set infinite constraints so it can size naturally
  clone.style.width = 'max-content';
  clone.style.height = 'max-content';
  clone.style.position = 'absolute';
  clone.style.visibility = 'hidden';
  clone.style.whiteSpace = 'nowrap'; // Prevents text wrapping if you want true max-length

  // Append to document temporarily to compute layout metrics
  document.body.appendChild(clone);
  
  // Measure natural dimensions
  const width = clone.scrollWidth;
  const height = clone.scrollHeight;

  // Clean up
  document.body.removeChild(clone);

  return { width, height };
}

const NAVBAR_ITEM_OBSERVER = new ResizeObserver((entries) => {
  for (const entry of entries) {
    const state = STATE.get(entry.target);
    
    // If there enough space, then the text is visible
    const isTextVisibleOld = state.isTextVisible;
    let isTextVisibleNew = entry.target.scrollWidth >= state.naturalWidth;
    
    if (!isTextVisibleNew && isTextVisibleOld) {
      // Else hide it if its too small
      for (const nav_item of entry.target.querySelectorAll(".navbar_item > a > span")) {
        nav_item.hidden = true
      }
    } else if (isTextVisibleNew && !isTextVisibleOld) {
      // Show the text like home, gallery, etc. If the navbar container is wide enough
      for (const nav_item of entry.target.querySelectorAll(".navbar_item > a > span")) {
        nav_item.hidden = false
      }
    }
    
    state.isTextVisible = isTextVisibleNew
  }
});

for (const container of document.querySelectorAll(".navbar_container")) {
  NAVBAR_ITEM_OBSERVER.observe(container)
  const { width } = getNaturalSize(container)
  
  STATE.set(container, {
    naturalWidth: width,
    isTextVisible: true
  })
}


