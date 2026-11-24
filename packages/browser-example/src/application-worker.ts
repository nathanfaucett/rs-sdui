import { startApplicationWorker } from "@rs-a2ui/web/worker";
import init, * as application from "./pkg/browser";

startApplicationWorker({ application, initialize: init });
