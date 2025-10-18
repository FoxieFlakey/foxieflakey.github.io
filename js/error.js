// A pretty basic error logging

const errorDiv = document.getElementById("error_div");

function doError(message) {
  message += "Stacktrace:\r\n";
  let stacktrace = new Error().stack;
  if (stacktrace === undefined || stacktrace === null) {
    stacktrace = "stack trace unavailable";
  }
  
  // Prepend tab on each line
  stacktrace = stacktrace.replace(/^/gm, '    ');
  message += stacktrace.toString();
  
  errorDiv.style.display = "";
  errorDiv.textContent += message;
}

export function error(messageRaw) {
  let message = "Error on client side: " + messageRaw.toString() + "\r\n";
  console.error(message);
  
  if (document.readyState === "complete") {
    doError(message)
  } else {
    document.addEventListener("load", () => doError(message), { once: true });
  }
}

