import { invoke } from "@tauri-apps/api/core";

import "./styles.css";

type BootstrapStatus = "ready";

interface WorkspaceBootstrap {
  protocolVersion: number;
  status: BootstrapStatus;
}

const bootstrapStatusElement =
  document.querySelector<HTMLElement>("#bootstrap-status");

if (!bootstrapStatusElement) {
  throw new Error("The bootstrap status element is missing.");
}

const bootstrapStatus: HTMLElement = bootstrapStatusElement;

async function verifyTrustedCoreBoundary(): Promise<void> {
  try {
    const result = await invoke<WorkspaceBootstrap>("workspace_bootstrap");
    bootstrapStatus.textContent = `Trusted core ${result.status} (protocol ${result.protocolVersion}).`;
  } catch (error) {
    bootstrapStatus.textContent = "Unable to reach the trusted core.";
    console.error("workspace_bootstrap command failed", error);
  }
}

void verifyTrustedCoreBoundary();
