// ============================================
// Message Content Component
// Markdown rendering with syntax highlighting
// ============================================

import { createEffect, createSignal, onMount, Show, For } from 'solid-js';
import { renderMarkdown, extractPlainText, hasMentions, getMentions } from '../../utils/markdown';
import type { FileAttachment } from '../../types/messages';
import { formatFileSize } from '../../utils/file';
import { cn } from '../../utils/cn';

// Icons
import {
  HiOutlineDocument,
  HiOutlinePhoto,
  HiOutlineFilm,
  HiOutlineMusicalNote,
  HiOutlineDocumentText,
  HiOutlineCodeBracket,
  HiOutlineArchiveBox,

} from 'solid-icons/hi';

// ============================================
// Types
// ============================================

interface MessageContentProps {
  content: string;
  files?: FileAttachment[];
  editedAt?: string;
  class?: string;
}

interface FilePreviewProps {
  file: FileAttachment;
}

// ============================================
// File Type Helpers
// ============================================

function getFileIcon(mimeType: string) {
  if (mimeType.startsWith('image/')) return HiOutlinePhoto;
  if (mimeType.startsWith('video/')) return HiOutlineFilm;
  if (mimeType.startsWith('audio/')) return HiOutlineMusicalNote;
  if (mimeType.includes('pdf')) return HiOutlineDocumentText;
  if (mimeType.includes('javascript') || mimeType.includes('json') || mimeType.includes('text')) return HiOutlineCodeBracket;
  if (mimeType.includes('zip') || mimeType.includes('tar') || mimeType.includes('rar') || mimeType.includes('7z')) return HiOutlineArchiveBox;
  return HiOutlineDocument;
}

function isImageFile(mimeType: string): boolean {
  return mimeType.startsWith('image/');
}

function isVideoFile(mimeType: string): boolean {
  return mimeType.startsWith('video/');
}

function isAudioFile(mimeType: string): boolean {
  return mimeType.startsWith('audio/');
}

// ============================================
// File Preview Component
// ============================================

function FilePreview(props: FilePreviewProps) {
  const icon = () => getFileIcon(props.file.mime_type);
  const isImage = () => isImageFile(props.file.mime_type);
  const isVideo = () => isVideoFile(props.file.mime_type);
  const isAudio = () => isAudioFile(props.file.mime_type);
  const IconComponent = icon();

  const handleDownload = () => {
    if (props.file.url) {
      const a = document.createElement('a');
      a.href = props.file.url;
      a.download = props.file.name;
      a.click();
    }
  };

  return (
    <div class="group/file relative">
      {/* Image Preview */}
      <Show when={isImage()}>
        <div class="relative inline-block max-w-sm rounded-lg overflow-hidden border border-border-1 hover:border-brand transition-colors">
          <img
            src={props.file.url}
            alt={props.file.name}
            class="max-w-full h-auto max-h-64 object-contain bg-bg-surface-2"
            loading="lazy"
          />
          <div class="absolute inset-0 bg-black/0 group-hover/file:bg-black/10 transition-colors" />
          <button
            onClick={handleDownload}
            class="absolute top-2 right-2 p-1.5 bg-bg-surface-1/90 rounded-md text-text-2 hover:text-text-1 opacity-0 group-hover/file:opacity-100 transition-opacity"
            title="Download"
          >
            ⬇️
          </button>
        </div>
      </Show>

      {/* Video Preview */}
      <Show when={isVideo()}>
        <div class="max-w-sm rounded-lg overflow-hidden border border-border-1">
          <video
            src={props.file.url}
            controls
            class="max-w-full max-h-64 bg-bg-surface-2"
            preload="metadata"
          />
          <div class="px-3 py-2 bg-bg-surface-1 border-t border-border-1 flex items-center justify-between">
            <span class="text-sm text-text-2 truncate flex-1">{props.file.name}</span>
            <button
              onClick={handleDownload}
              class="p-1.5 text-text-2 hover:text-text-1"
              title="Download"
            >
              ⬇️
            </button>
          </div>
        </div>
      </Show>

      {/* Audio Preview */}
      <Show when={isAudio()}>
        <div class="max-w-sm rounded-lg border border-border-1 p-3 bg-bg-surface-1">
          <div class="flex items-center gap-3 mb-2">
            <div class="w-10 h-10 rounded-full bg-brand/10 flex items-center justify-center">
              <HiOutlineMusicalNote size={20} class="text-brand" />
            </div>
            <div class="flex-1 min-w-0">
              <p class="text-sm font-medium text-text-1 truncate">{props.file.name}</p>
              <p class="text-xs text-text-3">{formatFileSize(props.file.size)}</p>
            </div>
            <button
              onClick={handleDownload}
              class="p-1.5 text-text-2 hover:text-text-1"
              title="Download"
            >
              ⬇️
            </button>
          </div>
          <audio src={props.file.url} controls class="w-full" />
        </div>
      </Show>

      {/* Generic File */}
      <Show when={!isImage() && !isVideo() && !isAudio()}>
        <div class="inline-flex items-center gap-3 px-3 py-2 rounded-lg border border-border-1 bg-bg-surface-1 hover:bg-bg-surface-2 hover:border-brand transition-colors cursor-pointer group/generic">
          <div class="w-10 h-10 rounded-lg bg-brand/10 flex items-center justify-center flex-shrink-0">
            <IconComponent size={20} class="text-brand" />
          </div>
          <div class="flex-1 min-w-0">
            <p class="text-sm font-medium text-text-1 truncate group-hover/generic:text-brand transition-colors">
              {props.file.name}
            </p>
            <p class="text-xs text-text-3">{formatFileSize(props.file.size)}</p>
          </div>
          <button
            onClick={handleDownload}
            class="p-1.5 text-text-2 hover:text-text-1"
            title="Download"
          >
            ⬇️
          </button>
        </div>
      </Show>
    </div>
  );
}

// ============================================
// Main Message Content Component
// ============================================

export default function MessageContent(props: MessageContentProps) {
  let contentRef: HTMLDivElement | undefined;
  const [renderedContent, setRenderedContent] = createSignal('');

  // Render markdown on mount and when content changes
  createEffect(() => {
    const html = renderMarkdown(props.content);
    setRenderedContent(html);
  });

  // Setup copy button handlers
  onMount(() => {
    if (!contentRef) return;

    const copyButtons = contentRef.querySelectorAll('.copy-btn');
    copyButtons.forEach(btn => {
      btn.addEventListener('click', () => {
        const code = btn.getAttribute('data-code');
        if (code) {
          navigator.clipboard.writeText(code);
          const originalText = btn.textContent;
          btn.textContent = 'Copied!';
          setTimeout(() => {
            btn.textContent = originalText;
          }, 2000);
        }
      });
    });
  });

  return (
    <div class={cn('space-y-2', props.class)}>
      {/* Markdown Content */}
      <div
        ref={contentRef}
        class="message-content text-text-1 text-sm leading-relaxed prose prose-sm max-w-none"
        innerHTML={renderedContent()}
      />

      {/* File Attachments */}
      <Show when={props.files && props.files.length > 0}>
        <div class="flex flex-wrap gap-2 mt-2">
          <For each={props.files}>
            {(file) => <FilePreview file={file} />}
          </For>
        </div>
      </Show>

      {/* Edited Indicator */}
      <Show when={props.editedAt}>
        <span class="text-xs text-text-3 italic">(edited)</span>
      </Show>
    </div>
  );
}

// ============================================
// Re-exports for convenience
// ============================================

export { extractPlainText, hasMentions, getMentions };
