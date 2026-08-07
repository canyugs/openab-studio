import { invoke } from "@tauri-apps/api/core";
import type { WorkspaceBootstrapResult } from "../../../schemas/generated/typescript/studio-protocol";

import "./styles.css";

const bootstrapStatusElement =
  document.querySelector<HTMLElement>("#bootstrap-status");

if (!bootstrapStatusElement) {
  throw new Error("The bootstrap status element is missing.");
}

const bootstrapStatus: HTMLElement = bootstrapStatusElement;

async function verifyTrustedCoreBoundary(): Promise<void> {
  try {
    const requestId = `req_${crypto.randomUUID()}`;
    const result = await invoke<WorkspaceBootstrapResult>(
      "workspace_bootstrap",
      { requestId },
    );
    bootstrapStatus.textContent = `Trusted core ${result.workspaceBootstrap.status} (protocol ${result.workspaceBootstrap.protocolVersion}).`;
  } catch (error) {
    bootstrapStatus.textContent = "Unable to reach the trusted core.";
    console.error("workspace_bootstrap command failed", error);
  }
}

void verifyTrustedCoreBoundary();
