<script setup lang="ts">
import { onMounted } from 'vue'
import { useChatStore } from '../../stores/chatStore'
import HistoryPanel from './HistoryPanel.vue'
import SelectorBar from './SelectorBar.vue'
import GatewayConfig from './GatewayConfig.vue'
import ReasoningToggle from './ReasoningToggle.vue'
import MessageList from './MessageList.vue'
import InputArea from './InputArea.vue'

const store = useChatStore()

onMounted(() => {
  store.loadAgents()
  store.loadModels()
  store.loadGatewayConfig()
  store.loadSessions()
})
</script>

<template>
  <div class="flex relative h-full">
    <HistoryPanel />

    <div class="flex overflow-hidden flex-col flex-1">
      <div class="flex justify-between items-center p-4 border-b border-dark-600">
        <SelectorBar />
        <div class="flex gap-2 items-center">
          <ReasoningToggle />
          <GatewayConfig />
        </div>
      </div>

      <MessageList />

      <InputArea />
    </div>
  </div>
</template>
