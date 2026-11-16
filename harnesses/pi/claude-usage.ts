// Runs the sense claude-usage at each step, so that the agent's inbox tells it how much of its
// Claude subscription's usage limits is used. Load it before the extension that takes the inbox
// in, so that what it delivers is taken in the same step.
// Does nothing unless AGENT_INBOX is set.

import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";

export default function (pi: ExtensionAPI) {
	if (!process.env.AGENT_INBOX) return;

	// A failing sense, such as for an expired login, must not stop the agent's step.
	async function sense() {
		await pi.exec("claude-usage", []).catch(() => undefined);
	}

	pi.on("before_agent_start", sense);
	pi.on("tool_execution_end", sense);
}
