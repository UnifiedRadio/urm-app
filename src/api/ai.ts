import type { AiConfig } from '../store/aiConfig'

const SYSTEM_PROMPT = `你是 OpenRadio 的无线电配置助手。
帮助用户配置业余无线电频道。始终提醒用户遵守当地法规。
如需输出频道配置，请使用 URC-v1 YAML 格式。
回答使用中文。`

export async function callAi(prompt: string, config: AiConfig): Promise<string> {
  if (!config.apiKey) throw new Error('未配置 AI API Key，请在设置中填写。')

  const res = await fetch(`${config.baseUrl}/chat/completions`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${config.apiKey}`,
    },
    body: JSON.stringify({
      model: config.model,
      messages: [
        { role: 'system', content: SYSTEM_PROMPT },
        { role: 'user', content: prompt },
      ],
      max_tokens: 1024,
      temperature: 0.7,
    }),
  })

  if (!res.ok) {
    const body = await res.text()
    throw new Error(`AI API 错误 ${res.status}: ${body}`)
  }

  const data = await res.json()
  const content = data.choices?.[0]?.message?.content
  if (!content) throw new Error('AI 返回内容为空')
  return content
}

export async function callAiRiskSummary(
  locationName: string,
  checklistItems: string[],
  officialLinks: string[],
  config: AiConfig,
): Promise<string> {
  const prompt = [
    `活动地点：${locationName || '未指定'}`,
    `确认清单状态：\n${checklistItems.map(t => `- ${t}`).join('\n')}`,
    officialLinks.length
      ? `官方参考链接：\n${officialLinks.map(u => `- ${u}`).join('\n')}`
      : '',
    '\n请根据以上信息生成一段简洁的活动安全风险摘要（100-200字），重点提示需要关注的安全事项。',
    '注意：仅供辅助参考，不作为安全决策依据。',
  ].filter(Boolean).join('\n')

  return callAi(prompt, config)
}
