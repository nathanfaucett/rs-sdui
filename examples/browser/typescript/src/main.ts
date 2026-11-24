import { mountA2ui } from "./mount-a2ui";
import { SharedWorkerTransport } from "./shared-worker-transport";
import "./style.css";

const connections = ["renderer-a", "renderer-b"].map((id) => {
  const target = document.getElementById(id);
  if (!target) throw new Error(`missing #${id} element`);
  return mountA2ui(target, "main", new SharedWorkerTransport()).connection;
});

window.addEventListener(
  "pagehide",
  () => {
    for (const connection of connections) connection.close();
  },
  { once: true }
);
