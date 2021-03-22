function show_report(report) {
    let report_div = document.getElementById("full_report");

    let content = String.fromCharCode.apply(null, report);

    document.getElementById("report_content").innerText = content.toString();

    report_div.style.display = "flex";
}

function close_report() {
    document.getElementById("full_report").style.display = "none";
}

document.getElementById("full_report").addEventListener("click", function() {close_report()});

document.getElementById("full_report_inner").addEventListener("click", e => e.stopPropagation());

window.addEventListener("keydown", function (event) {
  if (event.defaultPrevented) {
    return; // Do nothing if the event was already processed
  }

  switch (event.key) {
    case "Esc": // IE/Edge specific value
    case "Escape":
      close_report();
      break;
    default:
      return; // Quit when this doesn't handle the key event.
  }

  event.preventDefault();
}, true);
