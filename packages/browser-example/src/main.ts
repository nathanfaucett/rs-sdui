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
  const connectivity = document.getElementById(`connectivity-${id}`);
  if (!target || !(connectivity instanceof HTMLButtonElement)) {
    throw new Error(`missing ${id} renderer controls`);
  }

  let online = true;
  let connection = mountA2ui(target, "main", transport).connection;
  connectivity.addEventListener("click", () => {
    if (online) {
      connection.close();
      online = false;
      connectivity.textContent = "Go online";
      return;
    }

    target.replaceChildren();
    connection = mountA2ui(target, "main", transport).connection;
    online = true;
    connectivity.textContent = "Go offline";
  });
  window.addEventListener("pagehide", () => connection.close(), { once: true });
}
