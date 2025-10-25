import { createContentLoader } from 'vitepress'

export default createContentLoader('blog/posts/*.md', {
  transform(rawData) {
    return rawData
      .map(({ url, frontmatter }) => ({
        url,
        title: frontmatter.title,
        description: frontmatter.description,
        date: frontmatter.date,
        author: frontmatter.author,
        tags: frontmatter.tags
      }))
      .sort((a, b) => new Date(b.date) - new Date(a.date))
  }
})
