import { marked, type Renderer } from 'marked'
import hljs from 'highlight.js'

marked.setOptions({
  gfm: true,
  breaks: true,
})

const renderer: Partial<Renderer> = {
  code(token: { text: string; lang?: string }) {
    const language = token.lang && hljs.getLanguage(token.lang) ? token.lang : 'plaintext'
    const highlighted = hljs.highlight(token.text, { language }).value
    return `<pre class="hljs overflow-x-auto rounded-lg p-3 bg-dark-600 text-gray-200"><code class="language-${language}">${highlighted}</code></pre>`
  },
  codespan(token: { text: string }) {
    return `<code class="px-1 py-0.5 rounded bg-dark-600 text-claw-400 font-mono text-xs">${token.text}</code>`
  },
  link(token: { href: string; title?: string | null; text: string }) {
    const titleAttr = token.title ? ` title="${token.title}"` : ''
    return `<a href="${token.href}"${titleAttr} class="text-claw-400 hover:underline" target="_blank" rel="noopener noreferrer">${token.text}</a>`
  }
}

marked.use({ renderer })

export function renderMarkdown(content: string): string {
  if (!content) return ''
  try {
    return marked.parse(content) as string
  } catch {
    return content.replace(/\n/g, '<br>')
  }
}
