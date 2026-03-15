import { For, Show, createEffect, createMemo, createSignal } from 'solid-js';
import { HiOutlinePhone, HiOutlineUsers } from 'solid-icons/hi';
import Button from '@/components/ui/Button';
import { callsStore } from '@/stores/calls';
import { authStore } from '@/stores/auth';
import { toast } from '@/hooks/useToast';

function formatDuration(totalSeconds: number): string {
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`;
}

function RemoteStreamTile(props: { id: string; stream: MediaStream; hasVideo: boolean }) {
  let videoRef: HTMLVideoElement | undefined;
  let audioRef: HTMLAudioElement | undefined;

  createEffect(() => {
    if (props.hasVideo && videoRef) {
      videoRef.srcObject = props.stream;
    }
    if (!props.hasVideo && audioRef) {
      audioRef.srcObject = props.stream;
    }
  });

  return (
    <div class="rounded-lg border border-border-1 bg-bg-surface-2 p-2">
      <Show
        when={props.hasVideo}
        fallback={
          <div class="flex items-center justify-between gap-2 text-xs text-text-3">
            <span>Remote audio stream</span>
            <audio ref={audioRef} autoplay />
          </div>
        }
      >
        <video
          ref={videoRef}
          autoplay
          playsinline
          class="h-28 w-full rounded-md bg-black object-cover"
        />
      </Show>
    </div>
  );
}

export default function ActiveCallOverlay() {
  let localVideoRef: HTMLVideoElement | undefined;
  const [isRinging, setIsRinging] = createSignal(false);

  const session = () => callsStore.activeSession();
  const localVideoStream = createMemo(() => {
    const local = session()?.localStream;
    if (!local) return null;
    return local.getVideoTracks().length > 0 ? local : null;
  });

  createEffect(() => {
    const stream = localVideoStream();
    if (!stream || !localVideoRef) return;
    localVideoRef.srcObject = stream;
  });

  const durationLabel = createMemo(() => formatDuration(callsStore.durationSeconds()));
  const myUserId = () => authStore.user()?.id || '';
  const participants = createMemo(() =>
    [...callsStore.participants()].sort((left, right) => {
      const leftIsHost = left.userId === session()?.call.hostId;
      const rightIsHost = right.userId === session()?.call.hostId;
      if (leftIsHost === rightIsHost) return left.displayName?.localeCompare(right.displayName || '') || 0;
      return leftIsHost ? -1 : 1;
    })
  );

  const participantLabel = (participant: { displayName?: string; username?: string; userId: string }) =>
    participant.displayName || participant.username || participant.userId;

  const runHostAction = async (action: () => Promise<void>) => {
    try {
      await action();
    } catch (error) {
      toast.error(
        'Host action failed',
        error instanceof Error ? error.message : 'Unable to apply host control.'
      );
    }
  };

  const ringParticipants = async () => {
    setIsRinging(true);
    try {
      await callsStore.ringCurrentCall();
      toast.success('Ring sent', 'Participants have been notified.');
    } catch (error) {
      toast.error(
        'Unable to ring participants',
        error instanceof Error ? error.message : 'Failed to notify participants.'
      );
    } finally {
      setIsRinging(false);
    }
  };

  return (
    <Show when={session()}>
      {(active) => (
        <div
          class={`fixed bottom-4 right-4 z-[85] w-[360px] rounded-xl border border-border-1 bg-bg-surface-1 shadow-2xl ${
            callsStore.isExpanded() ? 'max-h-[80vh] overflow-hidden' : ''
          }`}
        >
          <div class="flex items-center justify-between border-b border-border-1 px-4 py-3">
            <div class="min-w-0">
              <p class="truncate text-sm font-semibold text-text-1">Active Call</p>
              <p class="text-xs text-text-3">
                {callsStore.participants().length} participants • {durationLabel()}
              </p>
            </div>
            <div class="flex items-center gap-2 text-text-3">
              <HiOutlinePhone size={16} />
              <HiOutlineUsers size={16} />
            </div>
          </div>

          <div class="space-y-3 p-3">
            <Show when={localVideoStream()}>
              <div class="rounded-lg border border-border-1 bg-bg-surface-2 p-2">
                <p class="mb-1 text-xs text-text-3">You</p>
                <video
                  ref={localVideoRef}
                  autoplay
                  muted
                  playsinline
                  class="h-28 w-full rounded-md bg-black object-cover"
                />
              </div>
            </Show>

            <Show when={active().remoteStreams.length > 0}>
              <div class="space-y-2">
                <p class="text-xs font-medium text-text-2">Remote media</p>
                <div class="max-h-48 space-y-2 overflow-y-auto pr-1">
                  <For each={active().remoteStreams}>
                    {(entry) => (
                      <RemoteStreamTile
                        id={entry.id}
                        stream={entry.stream}
                        hasVideo={entry.hasVideo}
                      />
                    )}
                  </For>
                </div>
              </div>
            </Show>

            <div class="flex items-center gap-2">
              <Button
                size="sm"
                variant={callsStore.isMuted() ? 'danger' : 'secondary'}
                onClick={() => {
                  void callsStore.toggleMute();
                }}
              >
                {callsStore.isMuted() ? 'Unmute' : 'Mute'}
              </Button>
              <Button
                size="sm"
                variant={callsStore.isHandRaised() ? 'primary' : 'secondary'}
                onClick={() => {
                  void callsStore.toggleHandRaised();
                }}
              >
                {callsStore.isHandRaised() ? 'Lower Hand' : 'Raise Hand'}
              </Button>
              <Show when={callsStore.canScreenShare()}>
                <Button
                  size="sm"
                  variant={callsStore.isScreenSharing() ? 'primary' : 'secondary'}
                  onClick={() => {
                    void callsStore.toggleScreenSharing();
                  }}
                >
                  {callsStore.isScreenSharing() ? 'Stop Share' : 'Share Screen'}
                </Button>
              </Show>
              <Button
                size="sm"
                variant="secondary"
                loading={isRinging()}
                onClick={() => {
                  void ringParticipants();
                }}
              >
                Ring
              </Button>
              <Show when={callsStore.isHost()}>
                <Button
                  size="sm"
                  variant="secondary"
                  onClick={() => {
                    void runHostAction(() => callsStore.hostMuteAllOthers());
                  }}
                >
                  Mute Others
                </Button>
              </Show>
            </div>

            <Show when={participants().length > 0}>
              <div class="rounded-lg border border-border-1 bg-bg-surface-2 p-2">
                <p class="mb-2 text-xs font-medium text-text-2">Participants</p>
                <div class="max-h-40 space-y-1 overflow-y-auto pr-1">
                  <For each={participants()}>
                    {(participant) => {
                      const isMe = () => participant.userId === myUserId();
                      const isHost = () => participant.userId === active().call.hostId;
                      return (
                        <div class="rounded border border-border-1/70 bg-bg-surface-1 px-2 py-1.5">
                          <div class="flex items-center gap-2">
                            <p class="min-w-0 flex-1 truncate text-xs font-medium text-text-1">
                              {participantLabel(participant)}
                            </p>
                            <Show when={isHost()}>
                              <span class="rounded bg-brand/15 px-1.5 py-0.5 text-[10px] font-semibold text-brand">
                                Host
                              </span>
                            </Show>
                            <Show when={isMe()}>
                              <span class="rounded bg-bg-app px-1.5 py-0.5 text-[10px] text-text-3">You</span>
                            </Show>
                            <Show when={participant.raisedHand > 0}>
                              <span class="rounded bg-warning/20 px-1.5 py-0.5 text-[10px] text-warning">
                                Hand
                              </span>
                            </Show>
                            <Show when={!participant.unmuted}>
                              <span class="rounded bg-danger/15 px-1.5 py-0.5 text-[10px] text-danger">
                                Muted
                              </span>
                            </Show>
                          </div>
                          <Show when={callsStore.isHost() && !isMe()}>
                            <div class="mt-1 flex items-center justify-end gap-1">
                              <button
                                type="button"
                                class="rounded border border-border-1 px-1.5 py-0.5 text-[10px] text-text-2 hover:bg-bg-surface-2"
                                onClick={() => {
                                  void runHostAction(() => callsStore.hostMuteSession(participant.sessionId));
                                }}
                              >
                                Mute
                              </button>
                              <Show when={participant.raisedHand > 0}>
                                <button
                                  type="button"
                                  class="rounded border border-border-1 px-1.5 py-0.5 text-[10px] text-text-2 hover:bg-bg-surface-2"
                                  onClick={() => {
                                    void runHostAction(() => callsStore.hostLowerHandSession(participant.sessionId));
                                  }}
                                >
                                  Lower Hand
                                </button>
                              </Show>
                              <button
                                type="button"
                                class="rounded border border-danger/40 px-1.5 py-0.5 text-[10px] text-danger hover:bg-danger/10"
                                onClick={() => {
                                  void runHostAction(() => callsStore.hostRemoveSession(participant.sessionId));
                                }}
                              >
                                Remove
                              </button>
                            </div>
                          </Show>
                        </div>
                      );
                    }}
                  </For>
                </div>
              </div>
            </Show>

            <div class="flex items-center justify-end gap-2 border-t border-border-1 pt-2">
              <Show when={callsStore.isHost()}>
                <Button
                  size="sm"
                  variant="danger"
                  onClick={() => {
                    void callsStore.endCurrentCall();
                  }}
                >
                  End Call
                </Button>
              </Show>
              <Button
                size="sm"
                variant="secondary"
                onClick={() => {
                  void callsStore.leaveCurrentCall();
                }}
              >
                Leave
              </Button>
            </div>
          </div>
        </div>
      )}
    </Show>
  );
}
