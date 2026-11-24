import { mountA2ui } from "./mount-a2ui";
import { SharedWorkerTransport } from "./shared-worker-transport";
import "./style.css";

for (const id of ["renderer-a", "renderer-b"]) {
  const target = document.getElementById(id);
  const reconnect = document.getElementById(`reconnect-${id}`);
  if (!target || !(reconnect instanceof HTMLButtonElement)) {
    throw new Error(`missing ${id} renderer controls`);
  }

  let connection = mountA2ui(target, "main", new SharedWorkerTransport()).connection;
  reconnect.addEventListener("click", () => {
    connection.close();
    target.replaceChildren();
    connection = mountA2ui(target, "main", new SharedWorkerTransport()).connection;
  });
  window.addEventListener("pagehide", () => connection.close(), { once: true });
}
