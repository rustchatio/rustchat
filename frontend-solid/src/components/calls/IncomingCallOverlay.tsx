import { Show, createEffect, createMemo, createSignal, onCleanup } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import Button from '@/components/ui/Button';
import { HiOutlinePhone, HiOutlinePhoneXMark } from 'solid-icons/hi';
import { callsStore } from '@/stores/calls';
import { channelStore, fetchChannel } from '@/stores/channels';
import { toast } from '@/hooks/useToast';

type JoinMode = 'voice' | 'video';

function formatRingingDuration(receivedAt: number, now: number): string {
  const seconds = Math.max(0, Math.floor((now - receivedAt) / 1000));
  if (seconds < 60) {
    return `${seconds}s`;
  }

  const minutes = Math.floor(seconds / 60);
  const remainder = seconds % 60;
  return `${minutes}m ${String(remainder).padStart(2, '0')}s`;
}

export default function IncomingCallOverlay() {
  const navigate = useNavigate();
  const incoming = () => callsStore.incomingCall();
  const [now, setNow] = createSignal(Date.now());

  const channelLabel = createMemo(() => {
    const call = incoming();
    if (!call) return 'Channel call';

    const stored = channelStore.getChannel(call.channelId);
    if (stored) {
      return stored.display_name || stored.name;
    }

    if (call.channelName && call.channelName.trim()) {
      return call.channelName;
    }

    return `Channel ${call.channelId.slice(0, 8)}`;
  });

  const ringingDuration = createMemo(() => {
    const call = incoming();
    if (!call) return '';
    return formatRingingDuration(call.receivedAt, now());
  });

  createEffect(() => {
    const call = incoming();
    if (!call) return;

    const timer = window.setInterval(() => setNow(Date.now()), 1000);
    onCleanup(() => window.clearInterval(timer));
  });

  createEffect(() => {
    const call = incoming();
    if (!call) return;

    if (!channelStore.getChannel(call.channelId)) {
      void fetchChannel(call.channelId).catch(() => {
        // Non-blocking channel-name enrichment.
      });
    }
  });

  const accept = async (mode: JoinMode) => {
    const call = incoming();
    if (!call) return;

    try {
      await callsStore.acceptIncomingCall(mode);
      navigate(`/channels/${call.channelId}`);
    } catch (error) {
      toast.error(
        'Unable to join call',
        error instanceof Error ? error.message : 'Unexpected error while joining call.'
      );
    }
  };

  const decline = () => {
    void callsStore.dismissIncomingCallRemote();
  };

  return (
    <Show when={incoming()}>
      {(call) => (
        <div class="fixed right-4 top-16 z-[95] w-[360px] rounded-xl border border-border-1 bg-bg-surface-1 p-4 shadow-2xl animate-fade-in">
          <div class="flex items-start gap-3">
            <div class="mt-0.5 flex h-10 w-10 items-center justify-center rounded-full bg-brand/10 text-brand">
              <HiOutlinePhone size={20} class="animate-pulse" />
            </div>
            <div class="min-w-0 flex-1">
              <p class="text-xs uppercase tracking-wide text-text-3">Incoming call</p>
              <h3 class="mt-1 truncate text-base font-semibold text-text-1">
                {call().senderName ? `${call().senderName} is calling` : 'Channel call is ringing'}
              </h3>
              <p class="mt-1 truncate text-sm text-text-2">{channelLabel()}</p>
              <p class="mt-0.5 text-xs text-text-3">Ringing for {ringingDuration()}</p>
            </div>
          </div>

          <div class="mt-4 flex items-center justify-end gap-2">
            <Button
              size="sm"
              variant="secondary"
              onClick={decline}
              aria-label="Decline incoming call"
            >
              <HiOutlinePhoneXMark size={16} />
              Decline
            </Button>
            <Button
              size="sm"
              variant="secondary"
              onClick={() => void accept('voice')}
              aria-label="Join incoming voice call"
            >
              <HiOutlinePhone size={16} />
              Voice
            </Button>
            <Button
              size="sm"
              variant="primary"
              onClick={() => void accept('video')}
              aria-label="Join incoming video call"
            >
              <HiOutlinePhone size={16} />
              Video
            </Button>
          </div>
        </div>
      )}
    </Show>
  );
}
