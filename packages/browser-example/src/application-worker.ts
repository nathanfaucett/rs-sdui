import { startApplicationWorker } from "@sdui/web/worker";
import init, * as application from "./pkg/browser";

startApplicationWorker({ application, initialize: init });
