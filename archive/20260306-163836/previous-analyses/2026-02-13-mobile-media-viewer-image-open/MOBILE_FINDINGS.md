# Mobile Findings (Mattermost Mobile)

## Viewer Behavior Relevant to Bug

1. Gallery image renderer requires valid dimensions.
- Gallery computes target size with `calculateDimensions(item.height, item.width, ...)`: `../mattermost-mobile/app/screens/gallery/renderers/image/index.tsx:37` to `../mattermost-mobile/app/screens/gallery/renderers/image/index.tsx:43`.
- If dimensions are missing/falsy, `calculateDimensions` returns `{height: 0, width: 0}`: `../mattermost-mobile/app/utils/images/index.ts:19` to `../mattermost-mobile/app/utils/images/index.ts:23`.

2. Attachment thumbnails can still appear when dimensions are bad.
- List rendering uses preview/file URLs (`buildFilePreviewUrl`/`buildFileUrl`) independent of gallery layout sizing: `../mattermost-mobile/app/hooks/files.ts:80`.
- This explains symptom pattern: thumbnail visible in post list, full-screen gallery blank.

## Contract-Relevant Conclusion

Rustchat must provide non-zero image dimensions in `metadata.files` for attachments, otherwise mobile gallery receives valid URI but renders an image with zero layout size.
