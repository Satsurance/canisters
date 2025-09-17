<template>
  <nav class="bg-white shadow-lg">
    <div class="container mx-auto px-4">
      <div class="relative flex h-16 items-center">
        <div class="flex justify-between items-center">
          <router-link class="flex items-center gap-4 cursor-pointer" to="/">
            <div class="w-10 h-10 bg-yellow-500 backdrop-blur-sm bg-opacity-90 rounded-xl flex items-center justify-center shadow-md">
              <span class="text-white font-bold text-xl">S</span>
            </div>
            <h1 class="text-slate-800 text-xl font-semibold tracking-tight">
              Satsurance
            </h1>
          </router-link>
        </div>

        <div class="hidden lg:flex absolute left-1/2 -translate-x-1/2">
          <div class="flex items-center space-x-8">
            <router-link
                to="/faucet"
                class="text-gray-600 hover:text-yellow-500"
                active-class="text-blue-600"
            >
              Faucet
            </router-link>
            <!-- Dropdown menu for Covers -->
            <div class="relative group">
              <!-- <div class="flex items-center font-medium text-gray-600 hover:text-yellow-500 space-x-1 py-4" >
                <span>Insurance</span>
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="h-4 w-4 transition-transform"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                >
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                </svg>
              </div> -->
              <!-- Added invisible bridge and adjusted positioning -->
              <div class="absolute hidden group-hover:block top-[calc(100%-16px)] pt-4 w-36">
                <div class="bg-white shadow-lg rounded-lg py-2">
                  <router-link
                      to="/cover_buy"
                      class="block px-4 py-2 text-gray-600 hover:text-yellow-500 hover:bg-gray-50"
                      active-class="text-blue-600"
                  >
                    Markets
                  </router-link>
                  <router-link
                      to="/cover_my"
                      class="block px-4 py-2 text-gray-600 hover:text-yellow-500 hover:bg-gray-50"
                      active-class="text-blue-600"
                  >
                    Purchases
                  </router-link>
                  <router-link
                      to="/cover"
                      class="block px-4 py-2 text-gray-600 hover:text-yellow-500 hover:bg-gray-50"
                      active-class="text-blue-600"
                  >
                    Custom
                  </router-link>
                </div>
              </div>
            </div>
            <router-link
                to="/stake"
                class="text-gray-600 hover:text-yellow-500"
                active-class="text-blue-600"
            >
              Underwrite
            </router-link>
            
            <!-- <a
                href="https://satsurance.gitbook.io/satsurance"
                target="_blank"
                class="text-gray-600 hover:text-yellow-500"
                active-class="text-blue-600"
            >
              Learn More
            </a> -->
          </div>
        </div>

        <div class="hidden lg:flex ml-auto items-center justify-end space-x-4">
          <ConnectWallet />
        </div>

        <div
            class="flex lg:hidden ml-auto items-center justify-end space-x-4"
            @click="toggleMobileNav"
        >
          <div class="p-2 cursor-pointer">
            <svg
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                stroke-width="1.5"
                stroke="currentColor"
                class="size-6"
            >
              <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  d="M3.75 5.25h16.5m-16.5 4.5h16.5m-16.5 4.5h16.5m-16.5 4.5h16.5"
              />
            </svg>
          </div>
        </div>
      </div>
    </div>
  </nav>

  <nav
      v-if="isMobileNavOpen"
      ref="mobileNav"
      class="fixed right-0 top-0 w-[85%] sm:w-3/4 h-screen bg-white lg:hidden z-[999] shadow-lg"
  >
    <div class="flex flex-col w-full h-full justify-evenly items-center">
      <div class="flex justify-between items-center">
        <router-link class="flex items-center gap-4 cursor-pointer" to="/">
          <div class="w-10 h-10 bg-yellow-500 backdrop-blur-sm bg-opacity-90 rounded-xl flex items-center justify-center shadow-md">
            <span class="text-white font-bold text-xl">S</span>
          </div>
          <h1 class="text-slate-800 text-xl font-semibold tracking-tight">
            Satsurance
          </h1>
        </router-link>
      </div>
      <div class="flex flex-col w-full items-center">
        <!-- Mobile Covers dropdown -->
        <!-- <div class="w-full text-center">
          <button
              @click="toggleMobileCoversMenu"
              class="flex items-center justify-center w-full text-gray-600 hover:text-yellow-500 py-4 space-x-1"
          >
            <span>Insurance</span>
            <svg
                xmlns="http://www.w3.org/2000/svg"
                class="h-4 w-4 transition-transform"
                :class="{ 'rotate-180': isMobileCoversOpen }"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
            >
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
            </svg>
          </button>
          <div
              v-show="isMobileCoversOpen"
              class="bg-gray-50"
          >
            <router-link
                to="/cover_buy"
                class="block w-full text-gray-600 hover:text-yellow-500 py-3"
                active-class="text-blue-600"
            >
              Markets
            </router-link>
            <router-link
                to="/cover_my"
                class="block w-full text-gray-600 hover:text-yellow-500 py-3"
                active-class="text-blue-600"
            >
              Purchases
            </router-link>
            <router-link
                to="/cover"
                class="block w-full text-gray-600 hover:text-yellow-500 py-3"
                active-class="text-blue-600"
            >
              Custom
            </router-link>
          </div>
        </div> -->
        <router-link
            to="/stake"
            class="text-gray-600 hover:text-yellow-500 py-4"
            active-class="text-blue-600"
        >
          Underwrite
        </router-link>
        <router-link
            to="/faucet"
            class="text-gray-600 hover:text-yellow-500 py-4"
            active-class="text-blue-600"
        >
          Faucet
        </router-link>
        <!-- <a
            href="https://satsurance.gitbook.io/satsurance"
            target="_blank"
            class="text-gray-600 hover:text-yellow-500 py-4"
            active-class="text-blue-600"
        >
          Learn More
        </a> -->
      </div>
      <ConnectWallet />
    </div>
  </nav>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from "vue";
import { useRouter } from "vue-router";
import ConnectWallet from "../components/ConnectWallet.vue";

const isMobileNavOpen = ref(false);
const isMobileCoversOpen = ref(false);
const mobileNav = ref(null);
const router = useRouter();

const toggleMobileNav = (e) => {
  e.stopPropagation();
  isMobileNavOpen.value = !isMobileNavOpen.value;
};

const toggleMobileCoversMenu = () => {
  isMobileCoversOpen.value = !isMobileCoversOpen.value;
};

const handleClickOutside = (event) => {
  if (
      mobileNav.value &&
      !mobileNav.value.contains(event.target) &&
      isMobileNavOpen.value
  ) {
    isMobileNavOpen.value = false;
    isMobileCoversOpen.value = false;
  }
};

onMounted(() => {
  document.addEventListener("click", handleClickOutside);
  router.beforeEach((to, from, next) => {
    isMobileNavOpen.value = false;
    isMobileCoversOpen.value = false;
    next();
  });
});

onUnmounted(() => {
  document.removeEventListener("click", handleClickOutside);
});
</script>