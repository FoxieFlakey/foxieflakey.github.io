// load this for pages who want it to be able fit

new ResizeObserver((observedEntries) => {
  window.parent.postMessage({
    type: "foxie-resize-iframe",
    width: observedEntries[0].contentRect.width,
    height: observedEntries[0].contentRect.height
  }, '*');
}).observe(document.documentElement);

/**
 * 
 * @param {HTMLIFrameElement} frame 
 * @param {boolean} doWidth
 * @param {boolean} doHeight
 */
function foxie_autoFitIframe(frame) {
  var prevHeight = undefined;
  
  let doPoll = () => {
    let newHeight = frame.contentWindow.document.body.scrollHeight;
    
    // let newWidth = Math.trunc(event.data.width);
    // if (prevWidth != newWidth) {
    //   prevWidth = newWidth;
    // }
    
    if (prevHeight != newHeight) {
      frame.style.height = '';
      frame.style.height = newHeight + 'px';
      prevHeight = newHeight;
    }
  };
  
  setInterval(doPoll, 1000);
  
  window.addEventListener("message", (event) => {
    if (!(event.data.type === "foxie-resize-iframe")) {
      // Don't recognize this
      return;
    }
    
    if (event.source[0] != frame.contentWindow) {
      // Wrong iframe
      return;
    }
    
    doPoll();
  })
}


