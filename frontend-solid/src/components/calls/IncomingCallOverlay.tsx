import { Show } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import Button from '@/components/ui/Button';
import { HiOutlinePhone, HiOutlinePhoneXMark } from 'solid-icons/hi';
import { callsStore } from '@/stores/calls';
import { toast } from '@/hooks/useToast';

export default function IncomingCallOverlay() {
  const navigate = useNavigate();
  const incoming = () => callsStore.incomingCall();

  const accept = async () => {
    const call = incoming();
    if (!call) return;

    try {
      await callsStore.acceptIncomingCall('voice');
      navigate(`/channels/${call.channelId}`);
    } catch (error) {
      toast.error(
        'Unable to join call',
        error instanceof Error ? error.message : 'Unexpected error while joining call.'
      );
    }
  };

  const decline = () => {
    callsStore.dismissIncomingCall();
  };

  return (
    <Show when={incoming()}>
      {(call) => (
        <div class="fixed right-4 top-16 z-[95] w-[340px] rounded-xl border border-border-1 bg-bg-surface-1 p-4 shadow-2xl">
          <p class="text-xs uppercase tracking-wide text-text-3">Incoming call</p>
          <h3 class="mt-1 text-base font-semibold text-text-1">
            {call().senderName ? `${call().senderName} is calling` : 'Channel call is ringing'}
          </h3>
          <p class="mt-1 text-sm text-text-3">Channel ID: {call().channelId}</p>

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
            <Button size="sm" variant="primary" onClick={() => void accept()} aria-label="Join incoming call">
              <HiOutlinePhone size={16} />
              Join
            </Button>
          </div>
        </div>
      )}
    </Show>
  );
}

