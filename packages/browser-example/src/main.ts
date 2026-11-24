import { SharedWorkerTransport } from "@rs-a2ui/web";
import { mountA2ui } from "./mount-a2ui";
import "./style.css";

const transport = new SharedWorkerTransport(
  () =>
    new SharedWorker(new URL("./application-worker.ts", import.meta.url), {
      type: "module",
      name: "a2ui-todo-application",
    })
);

for (const id of ["renderer-a", "renderer-b"]) {
  const target = document.getElementById(id);
  const reconnect = document.getElementById(`reconnect-${id}`);
  if (!target || !(reconnect instanceof HTMLButtonElement)) {
    throw new Error(`missing ${id} renderer controls`);
  }

  let connection = mountA2ui(target, "main", transport).connection;
  reconnect.addEventListener("click", () => {
    connection.close();
    target.replaceChildren();
    connection = mountA2ui(target, "main", transport).connection;
  });
  window.addEventListener("pagehide", () => connection.close(), { once: true });
}
