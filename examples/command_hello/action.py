import json
import os
from pathlib import Path


def read_json_env(name):
    return json.loads(Path(os.environ[name]).read_text(encoding="utf-8"))


def main():
    input_payload = read_json_env("SKILLRUN_INPUT_JSON")
    context = read_json_env("SKILLRUN_CONTEXT_JSON")
    output_path = Path(os.environ["SKILLRUN_OUTPUT_JSON"])
    artifact_dir = Path(os.environ["SKILLRUN_ARTIFACT_DIR"])

    name = input_payload["name"]
    print(f"command adapter log: preparing greeting for {name}")

    receipt_name = "command-hello-receipt.md"
    artifact_dir.mkdir(parents=True, exist_ok=True)
    (artifact_dir / receipt_name).write_text(
        f"# Command Hello Receipt\n\n"
        f"- input name: {name}\n"
        f"- mode: {context['mode']}\n"
        f"- adapter: command\n",
        encoding="utf-8",
    )

    envelope = {
        "ok": True,
        "output": {
            "adapter": "command",
            "message": f"hello {name} from command adapter",
            "input_name": name,
            "context_mode": context["mode"],
        },
        "artifacts": [
            {
                "kind": "markdown",
                "path": receipt_name,
                "description": "Command adapter greeting receipt",
            }
        ],
        "display": {
            "markdown": f"hello {name} from command adapter",
        },
    }
    output_path.write_text(json.dumps(envelope, ensure_ascii=False, indent=2), encoding="utf-8")


if __name__ == "__main__":
    main()
