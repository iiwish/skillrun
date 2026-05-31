import { execFile } from "child_process";
import { promisify } from "util";
import { writeFileSync } from "fs";
import { join } from "path";

const execFileAsync = promisify(execFile);

export const inputSchema = {
  type: "object",
  required: ["chat_id", "content"],
  additionalProperties: false,
  properties: {
    chat_id: { type: "string", minLength: 1 },
    content: { type: "string", minLength: 1, maxLength: 500 },
    urgency: { type: "string", enum: ["low", "normal", "high"], default: "normal" },
    dry_run: { type: "boolean", default: true }
  }
};

export const outputSchema = {
  type: "object",
  required: ["sent", "chat_id", "content_preview", "mode", "artifact_path"],
  additionalProperties: false,
  properties: {
    sent: { type: "boolean" },
    chat_id: { type: "string" },
    content_preview: { type: "string" },
    mode: { type: "string", enum: ["dry_run", "live"] },
    artifact_path: { type: "string" }
  }
};

function checkSensitiveWords(content) {
  const patterns = [
    /sk-[a-zA-Z0-9]{20,}/i,
    /[A-Za-z0-9]{20,}-[A-Za-z0-9]{10,}/,
    /password\s*[:=]\s*\S+/i,
    /secret\s*[:=]\s*\S+/i,
    /private[_-]?key/i,
  ];
  for (const p of patterns) {
    if (p.test(content)) {
      throw new Error(`Sensitive content detected: content matches forbidden pattern (${p.toString()})`);
    }
  }
}

function getAllowlist() {
  const raw = process.env.LARK_NOTICE_ALLOWLIST || "";
  if (!raw) {
    throw new Error("LARK_NOTICE_ALLOWLIST is required and must contain at least one allowed chat_id");
  }
  return raw.split(",").map(s => s.trim()).filter(Boolean);
}

export function preflight(input, ctx) {
  const allowlist = getAllowlist();
  if (allowlist.length === 0) {
    throw new Error("LARK_NOTICE_ALLOWLIST is required and must contain at least one allowed chat_id");
  }

  if (!allowlist.includes(input.chat_id)) {
    throw new Error(`chat_id "${input.chat_id}" is not in the allowed list. Allowed: ${allowlist.join(", ")}`);
  }

  checkSensitiveWords(input.content);

  if (input.content.length > 500) {
    throw new Error("Content exceeds 500 characters");
  }
}

export async function run(input, ctx) {
  const artifactDir = process.env.SKILLRUN_ARTIFACT_DIR;
  const artifactName = "notice-receipt.md";
  const artifactPath = join(artifactDir, artifactName);

  const contentPreview = input.content.length > 100
    ? input.content.slice(0, 100) + "..."
    : input.content;

  if (input.dry_run) {
    const receipt =
      `# Lark Notice Preview (Dry Run)\n\n` +
      `- Target chat: ${input.chat_id}\n` +
      `- Urgency: ${input.urgency}\n` +
      `- Content preview: ${contentPreview}\n` +
      `- Mode: dry_run — no message was sent\n` +
      `- Timestamp: ${new Date().toISOString()}\n\n` +
      `To send for real, set dry_run=false.\n`;
    writeFileSync(artifactPath, receipt, "utf-8");

    return {
      sent: false,
      chat_id: input.chat_id,
      content_preview: contentPreview,
      mode: "dry_run",
      artifact_path: artifactName
    };
  }

  try {
    const { stdout, stderr } = await execFileAsync("lark-cli", [
      "im", "+messages-send",
      "--chat-id", input.chat_id,
      "--text", input.content,
      "--format", "json"
    ], { timeout: 25000 });

    const receipt =
      `# Lark Notice Receipt\n\n` +
      `- Target chat: ${input.chat_id}\n` +
      `- Urgency: ${input.urgency}\n` +
      `- Content preview: ${contentPreview}\n` +
      `- Mode: live\n` +
      `- Timestamp: ${new Date().toISOString()}\n\n` +
      `## CLI Output\n\n` +
      `\`\`\`json\n${stdout}\n\`\`\`\n\n` +
      (stderr ? `## Stderr\n\`\`\`\n${stderr}\n\`\`\`\n` : "");
    writeFileSync(artifactPath, receipt, "utf-8");

    return {
      sent: true,
      chat_id: input.chat_id,
      content_preview: contentPreview,
      mode: "live",
      artifact_path: artifactName
    };
  } catch (err) {
    const receipt =
      `# Lark Notice Failure\n\n` +
      `- Target chat: ${input.chat_id}\n` +
      `- Error: ${err.message}\n` +
      `- Timestamp: ${new Date().toISOString()}\n`;
    writeFileSync(artifactPath, receipt, "utf-8");

    throw new Error(`lark-cli failed: ${err.message}. Ensure lark-cli is installed and authenticated.`);
  }
}
