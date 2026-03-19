/**
 * OfflineBanner — sticky notification shown when the device is offline.
 *
 * Shows a status indicator with pending mutation count and a manual sync
 * button. Disappears automatically when network is restored and sync completes.
 */

import { Show } from "solid-js";
import { WifiOff, RefreshCw, CloudUpload } from "lucide-solid";
import { useI18n } from "~/i18n";

interface OfflineBannerProps {
  isOnline: boolean;
  isSyncing: boolean;
  pendingCount: number;
  onSync: () => void;
}

export function OfflineBanner(props: OfflineBannerProps) {
  const t = useI18n();

  return (
    <>
      {/* Offline warning */}
      <Show when={!props.isOnline}>
        <div class="fixed top-0 inset-x-0 z-[var(--z-toast)] flex items-center justify-center gap-2 bg-amber-500 text-white text-xs font-medium py-1.5 px-4 safe-area-pt">
          <WifiOff size={13} />
          <span>{t("common.mobile.offline") ?? "You are offline — changes will sync when reconnected."}</span>
          <Show when={props.pendingCount > 0}>
            <span class="ml-1 bg-white/20 rounded-full px-1.5 py-0.5">
              {props.pendingCount} {t("common.mobile.offlinePending") ?? "pending"}
            </span>
          </Show>
        </div>
      </Show>

      {/* Back online + syncing */}
      <Show when={props.isOnline && props.isSyncing}>
        <div class="fixed top-0 inset-x-0 z-[var(--z-toast)] flex items-center justify-center gap-2 bg-primary text-white text-xs font-medium py-1.5 px-4 safe-area-pt">
          <RefreshCw size={13} class="animate-spin" />
          <span>{t("common.mobile.syncing") ?? `Syncing ${props.pendingCount} pending changes…`}</span>
        </div>
      </Show>

      {/* Back online but queue not yet flushed — manual trigger */}
      <Show when={props.isOnline && !props.isSyncing && props.pendingCount > 0}>
        <div class="fixed top-0 inset-x-0 z-[var(--z-toast)] flex items-center justify-center gap-2 bg-income text-white text-xs font-medium py-1.5 px-4 safe-area-pt">
          <CloudUpload size={13} />
          <span>{t("common.mobile.syncReady") ?? `${props.pendingCount} offline changes ready to sync.`}</span>
          <button
            class="ml-2 underline cursor-pointer"
            onClick={props.onSync}
          >
            {t("common.mobile.syncNow") ?? "Sync now"}
          </button>
        </div>
      </Show>
    </>
  );
}
