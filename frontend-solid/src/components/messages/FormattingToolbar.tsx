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
    <div class="formatting-toolbar">
      {tools.map(tool => (
        <button
          class="format-button"
          onClick={() => props.onFormat(tool.id)}
          title={tool.title}
          type="button"
        >
          {tool.icon}
        </button>
      ))}
    </div>
  );
}
