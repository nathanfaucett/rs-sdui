import {
  type Connection,
  SharedWorkerTransport,
  type Transport,
  webSocketConnection,
} from "@sdui/web";
import { mountA2ui } from "./mount-a2ui";
import "./style.css";

const instances = document.getElementById("instances");
const addButton = document.getElementById("add-instance");
const template = document.getElementById("instance-template");
if (
  !instances ||
  !(addButton instanceof HTMLButtonElement) ||
  !(template instanceof HTMLTemplateElement)
) {
  throw new Error("missing application instance controls");
}

let nextInstance = 1;
const connections = new Set<() => void>();

function workerTransport(session: string): Transport {
  return new SharedWorkerTransport(
    () =>
      new SharedWorker(new URL("./application-worker.ts", import.meta.url), {
        type: "module",
        name: `a2ui-${session}`,
      })
  );
}

function websocketTransport(session: string): Transport {
  const url = new URL("/ws", window.location.href);
  url.protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
  url.searchParams.set("session", session);
  const socket = new WebSocket(url);
  return { connect: () => webSocketConnection(socket) };
}

function addInstance(): void {
  if (!instances || !(template instanceof HTMLTemplateElement)) {
    throw new Error("missing application instance template");
  }
  const content = template.content.cloneNode(true);
  if (!(content instanceof DocumentFragment)) throw new Error("invalid instance template");
  const section = content.querySelector("section");
  const title = content.querySelector("[data-title]");
  const backend = content.querySelector("[data-backend]");
  const session = content.querySelector("[data-session]");
  const settings = content.querySelector("[data-settings]");
  const connectivity = content.querySelector("[data-connectivity]");
  const remove = content.querySelector("[data-remove]");
  const target = content.querySelector("[data-target]");
  const status = content.querySelector("[data-status]");
  const statusDot = content.querySelector("[data-status-dot]");
  if (
    !section ||
    !title ||
    !(backend instanceof HTMLSelectElement) ||
    !(session instanceof HTMLInputElement) ||
    !(settings instanceof HTMLFormElement) ||
    !(connectivity instanceof HTMLButtonElement) ||
    !(remove instanceof HTMLButtonElement) ||
    !(target instanceof HTMLElement) ||
    !status ||
    !statusDot
  ) {
    throw new Error("incomplete application instance template");
  }

  title.textContent = `Instance ${nextInstance}`;
  nextInstance += 1;
  let connection: Connection | undefined;
  let online = false;

  const setOnline = (value: boolean) => {
    online = value;
    status.textContent = online ? "Connected" : "Offline";
    statusDot.classList.toggle("bg-emerald-400", online);
    statusDot.classList.toggle("bg-slate-600", !online);
    connectivity.textContent = online ? "Go offline" : "Go online";
  };
  const disconnect = () => {
    connection?.close();
    connection = undefined;
    setOnline(false);
  };
  const connect = () => {
    const id = session.value.trim();
    if (!id) {
      session.setCustomValidity("Enter a session ID.");
      session.reportValidity();
      return;
    }

    disconnect();
    target.replaceChildren();
    const transport = backend.value === "websocket" ? websocketTransport(id) : workerTransport(id);
    const mounted = mountA2ui(target, "main", transport);
    connection = mounted.connection;
    setOnline(true);
    void mounted.done.then(
      () => {
        if (connection === mounted.connection) disconnect();
      },
      (error: unknown) => {
        console.error("A2UI connection failed", error);
        if (connection === mounted.connection) disconnect();
      }
    );
  };

  session.addEventListener("input", () => session.setCustomValidity(""));
  settings.addEventListener("submit", (event) => {
    event.preventDefault();
    connect();
  });
  connectivity.addEventListener("click", () => {
    if (online) {
      disconnect();
    } else {
      connect();
    }
  });
  remove.addEventListener("click", () => {
    disconnect();
    connections.delete(disconnect);
    section.remove();
  });

  connections.add(disconnect);
  instances.append(content);
  connect();
}

addButton.addEventListener("click", addInstance);
window.addEventListener("pagehide", () => {
  for (const disconnect of connections) disconnect();
});

addInstance();
addInstance();
