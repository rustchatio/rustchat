interface FormattingToolbarProps {
  onFormat: (format: string) => void;
}

export function FormattingToolbar(props: FormattingToolbarProps) {
  const tools = [
    { id: 'bold', label: 'Bold', icon: 'B', title: 'Bold (Ctrl+B)' },
    { id: 'italic', label: 'Italic', icon: 'I', title: 'Italic (Ctrl+I)' },
    { id: 'code', label: 'Code', icon: '</>', title: 'Inline Code' },
    { id: 'codeblock', label: 'Code Block', icon: '{}', title: 'Code Block' },
    { id: 'quote', label: 'Quote', icon: '"', title: 'Quote' },
    { id: 'link', label: 'Link', icon: '🔗', title: 'Link (Ctrl+K)' },
  ];

  return (
    <div class="mb-2 flex flex-wrap items-center gap-1 border-b border-border-1 pb-2">
      {tools.map(tool => (
        <button
          class="inline-flex h-7 min-w-7 items-center justify-center rounded-md border border-border-1 px-2 text-xs font-medium text-text-2 transition-colors hover:bg-bg-surface-2 hover:text-text-1"
          onClick={() => props.onFormat(tool.id)}
          title={tool.title}
          type="button"
          aria-label={tool.label}
        >
          {tool.icon}
        </button>
      ))}
    </div>
  );
}
