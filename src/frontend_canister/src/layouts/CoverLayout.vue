# template section
<template>
  <div class="min-h-[85vh] pt-[5vh] bg-gray-50 py-8">
    <!-- Main Content Container -->
    <div class="max-w-6xl mx-auto px-4">
      <!-- Unified White Background Container -->
      <div class="bg-white rounded-lg shadow-lg p-8">
        <!-- Header Section -->
        <div class="mb-8">
          <div class="space-y-1">
            <h1
              class="text-4xl font-semibold text-gray-900 flex items-center gap-3"
            >
              <svg
                class="w-8 h-8 text-yellow-500"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"
                />
              </svg>
              Request an Insurance
            </h1>
            <p class="text-gray-500 text-left">
              Would you like to purchase an insurance? We're here to help you explore
              the future of decentralized trust.
            </p>
          </div>
        </div>

        <!-- Two Column Layout - Responsive for mobile -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6 md:gap-8">
          <!-- Left Column - Form -->
          <div class="text-start space-y-6">
            <div v-if="!isSubmitted && !hasError">
              <form @submit.prevent="handleSubmit" class="space-y-6">
                <div class="space-y-4">
                  <div>
                    <label
                      for="name"
                      class="block mb-2 text-sm font-medium text-gray-900"
                    >
                      Name
                    </label>
                    <input
                      type="text"
                      id="name"
                      v-model="formData.name"
                      class="form-input"
                      placeholder="Satoshi Nakamoto"
                      :disabled="isSubmitting"
                      required
                    />
                  </div>

                  <div>
                    <label
                      for="company"
                      class="block mb-2 text-sm font-medium text-gray-900"
                    >
                      Company
                    </label>
                    <input
                      type="text"
                      id="company"
                      v-model="formData.company"
                      class="form-input"
                      placeholder="Bitcoin"
                      :disabled="isSubmitting"
                      required
                    />
                  </div>

                  <div>
                    <label
                      for="phone"
                      class="block mb-2 text-sm font-medium text-gray-900"
                    >
                      Phone
                    </label>
                    <input
                      type="tel"
                      id="phone"
                      v-model="formData.phone"
                      class="form-input"
                      placeholder="+1 (123) 456-7890"
                      :disabled="isSubmitting"
                      required
                    />
                  </div>

                  <div>
                    <label
                      for="email"
                      class="block mb-2 text-sm font-medium text-gray-900"
                    >
                      Email
                    </label>
                    <input
                      type="email"
                      id="email"
                      v-model="formData.email"
                      class="form-input"
                      placeholder="satoshi@bitcoin.com"
                      :disabled="isSubmitting"
                      required
                    />
                  </div>
                </div>

                <div class="flex justify-center">
                  <button
                    type="submit"
                    :disabled="isSubmitting"
                    class="btn-primary transition-colors flex items-center gap-2"
                  >
                    <template v-if="isSubmitting">
                      <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke-width="1.5"
                        stroke="currentColor"
                        class="w-5 h-5 animate-spin"
                      >
                        <path
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0 3.181 3.183a8.25 8.25 0 0 0 13.803-3.7M4.031 9.865a8.25 8.25 0 0 1 13.803-3.7l3.181 3.182m0-4.991v4.99"
                        />
                      </svg>
                      Sending...
                    </template>
                    <template v-else> Apply for Insurance </template>
                  </button>
                </div>
              </form>
            </div>

            <!-- Success State -->
            <div v-if="isSubmitted" class="text-center space-y-4">
              <h1 class="text-3xl font-medium">We've received your request!</h1>
              <h2>Our team will contact you within 2 business days</h2>
              <img
                class="w-2/3 mx-auto"
                src="../assets/Success.gif"
                alt="Success"
              />
            </div>

            <!-- Error State -->
            <div v-if="hasError" class="text-center space-y-4">
              <div class="bg-yellow-50 border-l-4 border-yellow-400 p-4">
                <div class="ml-3">
                  <h1 class="text-2xl font-medium text-yellow-800">
                    Something went wrong
                  </h1>
                  <h2 class="text-yellow-700">
                    Please try submitting your request again later
                  </h2>
                </div>
              </div>
              <button
                @click="resetForm"
                class="bg-yellow-500 text-white px-6 py-2 rounded-lg hover:bg-yellow-600 transition-colors"
              >
                Try Again
              </button>
            </div>
          </div>

          <!-- Right Column - Helper Links -->
          <div class="flex flex-col space-y-8">
            <div class="space-y-6">
              <p class="font-bold text-center">
                These links may be helpful for you 🤓
              </p>
              <div class="space-y-4">
                <a
                  href="#"
                  class="flex items-center justify-center w-25 text-yellow-500 hover:text-yellow-600 hover:underline"
                >
                  <FileText class="h-6 w-6 mr-2" />
                  <span>See the policy</span>
                </a>

                <a
                  href="#"
                  class="flex items-center justify-center text-yellow-500 hover:text-yellow-600 hover:underline"
                >
                  <Shield class="h-6 w-6 mr-2" />
                  <span>Insurance partner</span>
                </a>

                <a
                  href="#"
                  class="flex items-center justify-center text-yellow-500 hover:text-yellow-600 hover:underline"
                >
                  <FileText class="h-6 w-6 mr-2" />
                  <span>Required application</span>
                </a>
              </div>
            </div>

            <div class="mt-auto">
              <img
                class="h-60 mx-auto"
                src="../assets/Insurance.gif"
                alt="Insurance"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive } from "vue";
import { FileText, Shield } from "lucide-vue-next";

const isSubmitting = ref(false);
const isSubmitted = ref(false);
const hasError = ref(false);

const formData = reactive({
  name: "",
  company: "",
  phone: "",
  email: "",
});

const resetForm = () => {
  hasError.value = false;
  isSubmitted.value = false;
  formData.name = "";
  formData.company = "";
  formData.phone = "";
  formData.email = "";
};

const handleSubmit = async () => {
  try {
    isSubmitting.value = true;
    hasError.value = false;

    const response = await fetch('/api/submit', {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(formData),
    });

    if (!response.ok) {
      throw new Error("Submission failed");
    }

    await new Promise((resolve) => setTimeout(resolve, 1000));
    isSubmitted.value = true;

    // Reset form after successful submission
    formData.name = "";
    formData.company = "";
    formData.phone = "";
    formData.email = "";
  } catch (error) {
    console.error("Error submitting application:", error);
    hasError.value = true;
  } finally {
    isSubmitting.value = false;
  }
};
</script>
