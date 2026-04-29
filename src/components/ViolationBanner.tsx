import { invoke } from "@tauri-apps/api/core";
import type { ViolationPayload } from "../types";
import { flagOf } from "../lib/countries";

export function ViolationBanner({
  v,
  onDismiss,
}: {
  v: ViolationPayload;
  onDismiss: () => void;
}) {
  const dismiss = () => {
    invoke("reset_violation_cooldown", {
      bundleId: v.bundleId,
      country: v.country,
    }).catch(() => {});
    onDismiss();
  };

  return (
    <div className="violation" onClick={dismiss} role="button" title="点击关闭">
      <span className="violation-icon">!</span>
      <span className="violation-text">
        <b>{v.appName}</b> 在 {flagOf(v.country)} {v.country ?? "?"} ·
        允许 {v.allowedCountries.join("/") || "未设"}
      </span>
      <span className="violation-close" aria-label="关闭">×</span>
    </div>
  );
}
