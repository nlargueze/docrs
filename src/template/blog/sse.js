/**
 * Server-side events
 */

if (!!window.EventSource) {
  const eventSource = new EventSource("/__sse__");

  // eventSource.onopen = function (event) {
  //   console.log("sse opened", event);
  // };

  // eventSource.onclose = function (event) {
  //   console.log("sse closed", event);
  // };

  eventSource.onerror = function (event) {
    console.error("sse error", event);
  };

  // NB: "message" event type is used as a special case when the server sends no event.
  eventSource.addEventListener("reload", function (event) {
    // console.log("sse message", event);
    console.log("i SSE:reload");
    window.location.reload();
  });
}
