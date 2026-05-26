import { defineStore } from 'pinia'
import { ref } from 'vue'
import { requireSupabase } from '../api/supabase'
import type { RiskPack, RiskPackWeather, RiskPackWeatherHour } from '../types/cloud'

const WEATHER_CACHE_TTL_MS = 30 * 60 * 1000

export const useRiskPackStore = defineStore('riskPack', () => {
  const current = ref<RiskPack | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  function defaultRiskPack(): RiskPack {
    return {
      location_name: '',
      latitude: null,
      longitude: null,
      activity_start: null,
      activity_end: null,
      weather_cache: null,
      official_links: [],
      checklist: [
        { id: '1', text: '队长已确认天气状况', checked: false },
        { id: '2', text: '队长已确认当地灾害预警', checked: false },
        { id: '3', text: '队长已确认交通路况', checked: false },
        { id: '4', text: '队长已查阅当地政府官方通知', checked: false },
        { id: '5', text: '所有成员已获知紧急联系信息', checked: false },
      ],
      captain_notes: '',
      ai_summary: null,
    }
  }

  async function fetchWeather(lat: number, lng: number): Promise<RiskPackWeather> {
    const now = new Date()
    if (
      current.value?.weather_cache &&
      new Date(current.value.weather_cache.fetched_at).getTime() + WEATHER_CACHE_TTL_MS > now.getTime()
    ) {
      return current.value.weather_cache
    }

    const url = new URL('https://api.open-meteo.com/v1/forecast')
    url.searchParams.set('latitude', String(lat))
    url.searchParams.set('longitude', String(lng))
    url.searchParams.set('hourly', 'temperature_2m,precipitation_probability,windspeed_10m')
    url.searchParams.set('forecast_days', '3')
    url.searchParams.set('timezone', 'auto')

    const res = await fetch(url.toString())
    if (!res.ok) throw new Error(`Open-Meteo 请求失败 (${res.status})`)
    const json = await res.json()

    const times: string[] = json.hourly?.time ?? []
    const temps: number[] = json.hourly?.temperature_2m ?? []
    const precip: number[] = json.hourly?.precipitation_probability ?? []
    const wind: number[] = json.hourly?.windspeed_10m ?? []

    const forecast: RiskPackWeatherHour[] = times.map((t, i) => ({
      time: t,
      temperature_2m: temps[i] ?? 0,
      precipitation_probability: precip[i] ?? 0,
      windspeed_10m: wind[i] ?? 0,
    }))

    return { fetched_at: now.toISOString(), source: 'open_meteo', forecast }
  }

  async function loadForActivity(activityId: string) {
    loading.value = true
    error.value = null
    try {
      const client = requireSupabase()
      const { data, error: err } = await client
        .from('activities')
        .select('risk_pack')
        .eq('id', activityId)
        .single()
      if (err) throw err
      current.value = (data?.risk_pack as RiskPack | null) ?? defaultRiskPack()
    } catch (e) {
      error.value = (e as Error).message
    } finally {
      loading.value = false
    }
  }

  async function saveRiskPack(activityId: string, pack: RiskPack) {
    loading.value = true
    error.value = null
    try {
      const client = requireSupabase()
      const { error: err } = await client
        .from('activities')
        .update({ risk_pack: pack })
        .eq('id', activityId)
      if (err) throw err
      current.value = pack
    } catch (e) {
      error.value = (e as Error).message
    } finally {
      loading.value = false
    }
  }

  return { current, loading, error, defaultRiskPack, fetchWeather, loadForActivity, saveRiskPack }
})
