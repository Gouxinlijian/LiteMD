/**
 * 标题转锚点 ID：英文/数字保留、空格转短横线、中文保留（URL 编码后可跳转）。
 * 该函数同时用于渲染锚点与 TOC 生成，两处必须保持一致。
 */
export function slugify(text) {
  const str = String(text || '')
    .trim()
    .toLowerCase()
    .replace(/[`~!@#$%^&*()_|+\-=?;:'",.<>{}[\]\\/]/g, '')
    .replace(/\s+/g, '-')
  return str || 'heading'
}
