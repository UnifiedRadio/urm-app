import { ref } from 'vue'

export function useAsync() {
  const loading = ref(false)
  const error = ref('')

  async function run<T>(fn: () => Promise<T>): Promise<T> {
    loading.value = true
    error.value = ''
    try {
      return await fn()
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  function clearError() {
    error.value = ''
  }

  return { loading, error, run, clearError }
}
