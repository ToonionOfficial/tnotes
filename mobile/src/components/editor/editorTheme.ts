import { BridgeExtension, defaultEditorTheme, type EditorTheme } from "@10play/tentap-editor"

export const EDITOR_BACKGROUND = "#141318"
export const EDITOR_FOREGROUND = "#E6E1E9"
export const EDITOR_MUTED = "#8E8C99"
export const EDITOR_ACCENT = "#CABEFF"
export const EDITOR_CARD = "#201F24"
export const EDITOR_BORDER = "#302E36"

export const noteEditorCss = `
  * {
    box-sizing: border-box;
    -webkit-tap-highlight-color: transparent;
  }

  html, body {
    background-color: ${EDITOR_BACKGROUND} !important;
    color: ${EDITOR_FOREGROUND} !important;
    font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    font-size: 18px;
    line-height: 1.55;
    margin: 0;
    padding: 0;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    caret-color: ${EDITOR_ACCENT};
    min-height: 100vh;
  }

  ::selection {
    background-color: rgba(202, 190, 255, 0.35);
  }

  .ProseMirror {
    outline: none;
    min-height: 100vh;
    padding: 24px 20px 180px 20px !important;
    word-break: break-word;
    color: ${EDITOR_FOREGROUND} !important;
    background-color: ${EDITOR_BACKGROUND} !important;
  }

  .ProseMirror p,
  .ProseMirror span,
  .ProseMirror div,
  .ProseMirror li {
    color: ${EDITOR_FOREGROUND} !important;
  }

  /* Placeholder suppressed */
  .is-editor-empty:first-child::before,
  .ProseMirror p.is-editor-empty:first-child::before {
    display: none !important;
    content: none !important;
  }

  /* Headings */
  h1 {
    font-size: 34px;
    font-weight: 700;
    line-height: 1.22;
    margin-top: 20px;
    margin-bottom: 10px;
    color: #FFFFFF !important;
    letter-spacing: -0.6px;
  }

  h2 {
    font-size: 26px;
    font-weight: 700;
    line-height: 1.26;
    margin-top: 18px;
    margin-bottom: 8px;
    color: #F4F0F7 !important;
    letter-spacing: -0.4px;
  }

  h3 {
    font-size: 21px;
    font-weight: 600;
    line-height: 1.32;
    margin-top: 16px;
    margin-bottom: 6px;
    color: ${EDITOR_FOREGROUND} !important;
    letter-spacing: -0.2px;
  }

  p {
    margin-top: 0;
    margin-bottom: 12px;
    font-size: 18px;
    line-height: 1.55;
  }

  /* Lists */
  ul, ol {
    padding-left: 24px;
    margin-top: 4px;
    margin-bottom: 12px;
  }

  li {
    margin-bottom: 4px;
    font-size: 18px;
    line-height: 1.55;
  }

  li p {
    margin-bottom: 0;
    font-size: 18px;
    line-height: 1.55;
  }

  /* Task / Checklist */
  ul[data-type="taskList"] {
    list-style: none;
    padding: 0;
    margin: 8px 0;
  }

  ul[data-type="taskList"] li {
    display: flex;
    align-items: flex-start;
    margin-bottom: 8px;
  }

  ul[data-type="taskList"] li > label {
    flex: 0 0 auto;
    margin-right: 12px;
    margin-top: 2px;
    user-select: none;
    cursor: pointer;
  }

  ul[data-type="taskList"] li > label > input[type="checkbox"] {
    appearance: none;
    -webkit-appearance: none;
    width: 22px;
    height: 22px;
    border: 2px solid ${EDITOR_MUTED};
    border-radius: 50%;
    outline: none;
    background-color: transparent;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    position: relative;
    vertical-align: middle;
    margin: 0;
    transition: all 0.2s ease;
  }

  ul[data-type="taskList"] li > label > input[type="checkbox"]:checked {
    background-color: ${EDITOR_ACCENT};
    border-color: ${EDITOR_ACCENT};
  }

  ul[data-type="taskList"] li > label > input[type="checkbox"]:checked::after {
    content: '';
    display: block;
    width: 5px;
    height: 10px;
    border: solid #32285F;
    border-width: 0 2.5px 2.5px 0;
    transform: rotate(45deg);
    position: absolute;
    top: 2px;
    left: 7px;
  }

  ul[data-type="taskList"] li[data-checked="true"] > div > p {
    text-decoration: line-through;
    color: ${EDITOR_MUTED} !important;
  }

  ul[data-type="taskList"] li > div {
    flex: 1 1 auto;
  }

  /* Blockquote */
  blockquote {
    border-left: 3.5px solid ${EDITOR_ACCENT};
    padding-left: 16px;
    margin-left: 0;
    margin-right: 0;
    margin-top: 12px;
    margin-bottom: 12px;
    color: #C6C2CD !important;
    font-style: italic;
    font-size: 18px;
    line-height: 1.55;
  }

  /* Code Inline & Block */
  code {
    background-color: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 5px;
    padding: 2px 7px;
    font-family: "SF Mono", Menlo, Monaco, Consolas, "Courier New", monospace;
    font-size: 15.5px;
    color: ${EDITOR_ACCENT} !important;
  }

  pre {
    background-color: ${EDITOR_CARD};
    border: 1px solid ${EDITOR_BORDER};
    border-radius: 10px;
    padding: 14px 16px;
    margin: 12px 0;
    overflow-x: auto;
  }

  pre code {
    background-color: transparent;
    border: none;
    padding: 0;
    color: ${EDITOR_FOREGROUND} !important;
    font-size: 15px;
    line-height: 1.5;
  }

  /* Links */
  a {
    color: ${EDITOR_ACCENT} !important;
    text-decoration: underline;
    text-underline-offset: 3px;
  }

  /* Horizontal Rule */
  hr {
    border: none;
    border-top: 1px solid ${EDITOR_BORDER};
    margin: 20px 0;
  }
`

export const ThemeBridge = new BridgeExtension({
  forceName: "tnotes-custom-theme",
  extendCSS: noteEditorCss,
})

export const noteEditorTheme: EditorTheme = {
  toolbar: {
    ...defaultEditorTheme.toolbar,
    toolbarBody: {
      backgroundColor: "transparent",
      borderTopWidth: 0,
      borderBottomWidth: 0,
    },
    hidden: {
      display: "none",
    },
  },
  webview: {
    backgroundColor: EDITOR_BACKGROUND,
  },
  webviewContainer: {
    backgroundColor: EDITOR_BACKGROUND,
  },
}
