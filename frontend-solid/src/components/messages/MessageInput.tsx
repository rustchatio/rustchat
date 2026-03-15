import { createSignal, createEffect, Show, For } from 'solid-js';
import { authStore } from '@/stores/auth';
import { channelStore } from '@/stores/channels';
import { sendMessage } from '@/stores/messages';
import { generateUUID } from '@/utils/uuid';
// API calls use fetch directly
import { Button } from '@/components/ui/Button';
import { formatFileSize } from '@/utils/file';
import { EmojiPicker } from './EmojiPicker';
import { MentionsAutocomplete } from './MentionsAutocomplete';
import { SlashCommands } from './SlashCommands';
import { FormattingToolbar } from './FormattingToolbar';

interface FileAttachment {
  id: string;
  file: File;
  name: string;
  size: number;
  type: string;
  progress: number;
  uploaded: boolean;
  uploadedFileId?: string;
  error?: string;
}

interface MessageInputProps {
  channelId: string;
  threadId?: string;
  placeholder?: string;
  onSend?: () => void;
}

const MAX_MESSAGE_LENGTH = 4000;
const MAX_FILE_SIZE = 50 * 1024 * 1024; // 50MB

export function MessageInput(props: MessageInputProps) {
  const [message, setMessage] = createSignal('');
  const [isSending, setIsSending] = createSignal(false);
  const [showEmojiPicker, setShowEmojiPicker] = createSignal(false);
  const [showMentions, setShowMentions] = createSignal(false);
  const [showCommands, setShowCommands] = createSignal(false);
  const [mentionQuery, setMentionQuery] = createSignal('');
  const [commandQuery, setCommandQuery] = createSignal('');
  const [files, setFiles] = createSignal<FileAttachment[]>([]);
  const [isDragging, setIsDragging] = createSignal(false);
  const [textareaRef, setTextareaRef] = createSignal<HTMLTextAreaElement | null>(null);
  const [cursorPosition, setCursorPosition] = createSignal(0);
  const [sendError, setSendError] = createSignal<string | null>(null);
  
  const auth = authStore;
  const channels = channelStore;
  // Keep refs for now (used by slash commands/typing context and future behavior).
  void auth;
  void channels;

  // Auto-resize textarea
  createEffect(() => {
    const textarea = textareaRef();
    message(); // Track message for resize
    if (textarea) {
      textarea.style.height = 'auto';
      textarea.style.height = `${Math.min(textarea.scrollHeight, 400)}px`;
    }
  });

  const handleInput = (e: Event) => {
    const target = e.target as HTMLTextAreaElement;
    const value = target.value;
    setMessage(value);
    setCursorPosition(target.selectionStart || 0);

    // Check for @mentions
    const beforeCursor = value.slice(0, target.selectionStart);
    const mentionMatch = beforeCursor.match(/@([\w]*)$/);
    if (mentionMatch) {
      setMentionQuery(mentionMatch[1]);
      setShowMentions(true);
      setShowCommands(false);
    } else {
      setShowMentions(false);
    }

    // Check for slash commands
    if (value.startsWith('/') && !value.includes('\n')) {
      setCommandQuery(value.slice(1));
      setShowCommands(true);
      setShowMentions(false);
    } else {
      setShowCommands(false);
    }

    // Send typing indicator
    sendTypingIndicator();
  };

  const sendTypingIndicator = (() => {
    let lastSent = 0;
    return () => {
      const now = Date.now();
      if (now - lastSent > 3000) {
        lastSent = now;
        // Send via WebSocket
        const ws = (window as any).__websocket;
        if (ws && ws.readyState === WebSocket.OPEN) {
          ws.send(JSON.stringify({
            action: 'user_typing',
            channel_id: props.channelId,
            parent_id: props.threadId,
          }));
        }
      }
    };
  })();

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleSend = async () => {
    setSendError(null);

    const attachments = files();

    if (!message().trim() && attachments.length === 0) return;
    if (message().length > MAX_MESSAGE_LENGTH) return;

    setIsSending(true);

    try {
      // Upload files first
      const fileIds: string[] = [];
      const token = authStore.token;
      for (const attachment of attachments) {
        if (attachment.error) continue;

        if (attachment.uploaded && attachment.uploadedFileId) {
          fileIds.push(attachment.uploadedFileId);
          continue;
        }

        if (!attachment.uploaded) {
          const formData = new FormData();
          formData.append('file', attachment.file);
          
          const response = await fetch(`/api/v1/files?channel_id=${encodeURIComponent(props.channelId)}`, {
            method: 'POST',
            headers: token ? { Authorization: `Bearer ${token}` } : {},
            body: formData,
          });
          if (!response.ok) {
            const payload = (await response.json().catch(() => ({}))) as { message?: string };
            throw new Error(payload.message || `File upload failed for ${attachment.name}`);
          }
          const payload = (await response.json()) as { id: string };
          updateFileProgress(attachment.id, 100);
          
          fileIds.push(payload.id);
          markFileUploaded(attachment.id, payload.id);
        }
      }

      await sendMessage(props.channelId, message().trim(), props.threadId, fileIds);

      // Clear input
      setMessage('');
      setFiles([]);
      props.onSend?.();
    } catch (error) {
      console.error('Failed to send message:', error);
      setSendError(error instanceof Error ? error.message : 'Failed to send message');
    } finally {
      setIsSending(false);
    }
  };

  const insertEmoji = (emoji: string) => {
    const textarea = textareaRef();
    if (!textarea) return;

    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const text = message();
    const newText = text.slice(0, start) + emoji + text.slice(end);
    
    setMessage(newText);
    setShowEmojiPicker(false);
    
    // Restore cursor position
    setTimeout(() => {
      textarea.focus();
      textarea.setSelectionRange(start + emoji.length, start + emoji.length);
    }, 0);
  };

  const insertMention = (username: string) => {
    const textarea = textareaRef();
    if (!textarea) return;

    const beforeCursor = message().slice(0, cursorPosition());
    const afterMatch = beforeCursor.replace(/@([\w]*)$/, `@${username} `);
    const afterCursor = message().slice(cursorPosition());
    
    setMessage(afterMatch + afterCursor);
    setShowMentions(false);
    
    setTimeout(() => {
      textarea.focus();
      const newPos = afterMatch.length;
      textarea.setSelectionRange(newPos, newPos);
    }, 0);
  };

  const executeCommand = (command: string) => {
    const commandMap: Record<string, string> = {
      'shrug': '¯\\_(ツ)_/¯',
      'tableflip': '(╯°□°)╯︵ ┻━┻',
      'unflip': '┬─┬ ノ( ゜-゜ノ)',
    };

    const result = commandMap[command] || '';
    if (result) {
      setMessage(result);
    }
    setShowCommands(false);
  };

  const applyFormatting = (format: string) => {
    const textarea = textareaRef();
    if (!textarea) return;

    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const selectedText = message().slice(start, end);
    let formattedText = '';

    switch (format) {
      case 'bold':
        formattedText = `**${selectedText || 'bold text'}**`;
        break;
      case 'italic':
        formattedText = `_${selectedText || 'italic text'}_`;
        break;
      case 'code':
        formattedText = `\`${selectedText || 'code'}\``;
        break;
      case 'codeblock':
        formattedText = `\`\`\`\n${selectedText || 'code block'}\n\`\`\``;
        break;
      case 'quote':
        formattedText = `> ${selectedText || 'quote'}`;
        break;
      case 'link':
        formattedText = `[${selectedText || 'link text'}](url)`;
        break;
    }

    const newText = message().slice(0, start) + formattedText + message().slice(end);
    setMessage(newText);

    setTimeout(() => {
      textarea.focus();
      const newPos = start + formattedText.length;
      textarea.setSelectionRange(newPos, newPos);
    }, 0);
  };

  // File handling
  const handleFileSelect = (selectedFiles: FileList | null) => {
    if (!selectedFiles) return;

    const newFiles: FileAttachment[] = [];
    for (const file of Array.from(selectedFiles)) {
        if (file.size > MAX_FILE_SIZE) {
          newFiles.push({
            id: generateUUID(),
            file,
            name: file.name,
            size: file.size,
            type: file.type,
          progress: 0,
          uploaded: false,
          error: `File too large (max ${formatFileSize(MAX_FILE_SIZE)})`,
        });
        } else {
          newFiles.push({
            id: generateUUID(),
            file,
            name: file.name,
            size: file.size,
            type: file.type,
          progress: 0,
          uploaded: false,
        });
      }
    }

    setFiles([...files(), ...newFiles]);
  };

  const updateFileProgress = (id: string, progress: number) => {
    setFiles(files().map(f => f.id === id ? { ...f, progress } : f));
  };

  const markFileUploaded = (id: string, uploadedFileId: string) => {
    setFiles(
      files().map((f) =>
        f.id === id ? { ...f, uploaded: true, progress: 100, uploadedFileId } : f
      )
    );
  };

  const removeFile = (id: string) => {
    setFiles(files().filter(f => f.id !== id));
  };

  const handleDragOver = (e: DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  };

  const handleDragLeave = (e: DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
  };

  const handleDrop = (e: DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    handleFileSelect(e.dataTransfer?.files || null);
  };

  const messageLength = () => message().length;
  const isOverLimit = () => messageLength() > MAX_MESSAGE_LENGTH;

  return (
    <div
      class="message-input-container"
      classList={{ 'dragging': isDragging() }}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
    >
      <FormattingToolbar onFormat={applyFormatting} />
      
      {/* File attachments */}
      <Show when={files().length > 0}>
        <div class="file-attachments">
          <For each={files()}>
            {(file) => (
              <div class="file-attachment" classList={{ 'error': !!file.error }}>
                <div class="file-info">
                  <span class="file-name">{file.name}</span>
                  <span class="file-size">{formatFileSize(file.size)}</span>
                  {file.error && <span class="file-error">{file.error}</span>}
                </div>
                <Show when={!file.error}>
                  <div class="file-progress">
                    <div class="progress-bar" style={{ width: `${file.progress}%` }} />
                  </div>
                </Show>
                <button
                  class="remove-file"
                  onClick={() => removeFile(file.id)}
                  type="button"
                >
                  ×
                </button>
              </div>
            )}
          </For>
        </div>
      </Show>

      <div class="input-row">
        <textarea
          ref={setTextareaRef}
          value={message()}
          onInput={handleInput}
          onKeyDown={handleKeyDown}
          placeholder={props.placeholder || 'Write a message...'}
          disabled={isSending()}
          rows={1}
          class="message-textarea"
          classList={{ 'over-limit': isOverLimit() }}
        />
        
        <div class="input-actions">
          <input
            type="file"
            id="file-upload"
            multiple
            onChange={(e) => handleFileSelect(e.target.files)}
            style={{ display: 'none' }}
          />
          <label for="file-upload" class="file-button" title="Attach file">
            📎
          </label>
          
          <button
            class="emoji-button"
            onClick={() => setShowEmojiPicker(!showEmojiPicker())}
            type="button"
            title="Add emoji"
          >
            😀
          </button>
          
          <Button
            onClick={handleSend}
            disabled={isSending() || (!message().trim() && files().length === 0) || isOverLimit()}
            loading={isSending()}
            size="sm"
          >
            Send
          </Button>
        </div>
      </div>

      {/* Character counter */}
      <Show when={messageLength() > 500}>
        <div class="character-counter" classList={{ 'over-limit': isOverLimit() }}>
          {messageLength()}/{MAX_MESSAGE_LENGTH}
        </div>
      </Show>

      <Show when={sendError()}>
        <div class="mt-2 rounded-lg border border-danger/30 bg-danger/10 px-3 py-2 text-xs text-danger">
          {sendError()}
        </div>
      </Show>

      {/* Emoji Picker */}
      <Show when={showEmojiPicker()}>
        <EmojiPicker
          onSelect={insertEmoji}
          onClose={() => setShowEmojiPicker(false)}
        />
      </Show>

      {/* Mentions Autocomplete */}
      <Show when={showMentions()}>
        <MentionsAutocomplete
          query={mentionQuery()}
          channelId={props.channelId}
          onSelect={insertMention}
          onClose={() => setShowMentions(false)}
        />
      </Show>

      {/* Slash Commands */}
      <Show when={showCommands()}>
        <SlashCommands
          query={commandQuery()}
          onSelect={executeCommand}
          onClose={() => setShowCommands(false)}
        />
      </Show>

      {/* Drag overlay */}
      <Show when={isDragging()}>
        <div class="drag-overlay">
          <div class="drag-message">Drop files here to upload</div>
        </div>
      </Show>
    </div>
  );
}
