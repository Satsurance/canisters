<template>
  <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6 hover:shadow-md transition-shadow cursor-pointer"
    @click="$emit('click')">
    <div class="flex flex-col h-full">
      <!-- Project Name -->
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-lg font-semibold text-gray-900">{{ project.name }}</h3>
        <span class="px-2.5 py-0.5 text-xs font-medium rounded-full" :class="{
          'bg-green-100 text-green-800': project.category === 'Web3',
          'bg-blue-100 text-blue-800': project.category === 'Slashing',
          'bg-purple-100 text-purple-800': project.category === 'Bridges'
        }">
          {{ project.category }}
        </span>
      </div>

      <!-- Logo -->
      <div class="flex justify-center items-center py-4 sm:py-6">
        <img :src="project.logo" :alt="`${project.name} logo`"
          class="w-24 h-24 sm:w-28 md:w-32 sm:h-28 md:h-32 object-contain" />
      </div>

      <!-- Info -->
      <div class="mt-auto">
        <p class="text-gray-600 mb-4 text-sm">{{ project.description }}</p>

        <div class="space-y-2">
          <div class="flex justify-between text-sm">
            <span class="text-gray-500">Max Coverage:</span>
            <span class="font-medium">{{ Math.floor(project.maxCover * 10000) / 10000 }} BTC</span>
          </div>
          <div class="flex justify-between text-sm">
            <span class="text-gray-500">Premium Rate:</span>
            <span class="text-yellow-600 font-medium">{{ project.rate }}% per year</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
defineProps({
  project: {
    type: Object,
    required: true
  }
});
</script>