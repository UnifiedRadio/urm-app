import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

const STORAGE_KEY = 'urm_ai_config'

export interface AiConfig {
  baseUrl: string
  apiKey: string
  model: string
}

const DEFAULTS: AiConfig = {
  baseUrl: 'https://ahbb.m1in.com/v1',
  apiKey: 'ah-04ab9dfd5352710f8e918652fcdd44cef224f2013fca689e89005089a4853789',
  model: 'deepseek-v4-flash',
}

function load(): AiConfig {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) return { ...DEFAULTS, ...JSON.parse(raw) }
  } catch { /* ignore */ }
  return { ...DEFAULTS }
}

export const useAiConfigStore = defineStore('aiConfig', () => {
  const saved = load()
  const baseUrl = ref(saved.baseUrl)
  const apiKey = ref(saved.apiKey)
  const model = ref(saved.model)

  function save() {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({
      baseUrl: baseUrl.value,
      apiKey: apiKey.value,
      model: model.value,
    }))
  }

  function reset() {
    baseUrl.value = DEFAULTS.baseUrl
    apiKey.value = DEFAULTS.apiKey
    model.value = DEFAULTS.model
    save()
  }

  watch([baseUrl, apiKey, model], save)

  return { baseUrl, apiKey, model, save, reset }
})
