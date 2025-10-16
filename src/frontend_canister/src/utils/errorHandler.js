import { getCurrentNetwork, ICP_NETWORKS } from '../constants/icp.js';

/**
 * Checks if an error is a signature verification error from the Plug wallet
 * @param {Error} error - The error to check
 * @returns {boolean} - True if the error is a signature verification error
 */
export const isSignatureVerificationError = (error) => {
  if (!error) return false;

  const errorString = error.toString().toLowerCase();
  const errorMessage = (error.message || '').toLowerCase();

  return (
    errorString.includes('signature verification') ||
    errorString.includes('invalid certificate') ||
    errorMessage.includes('signature verification') ||
    errorMessage.includes('invalid certificate')
  );
};

export const isFetchRootKeyError = (error) => {
  if (!error) return false;
  const errorString = error.toString().toLowerCase();
  const errorMessage = (error.message || '').toLowerCase();
  return (
    errorString.includes('failed to fetch') ||
    errorString.includes('fetchrootkey') ||
    errorMessage.includes('failed to fetch') ||
    errorMessage.includes('fetchrootkey')
  );
};

export const isInvalidReadStateError = (error) => {
  if (!error) return false;
  const errorString = error.toString().toLowerCase();
  const errorMessage = (error.message || '').toLowerCase();
  return (
    errorString.includes('invalid read state request') ||
    errorMessage.includes('invalid read state request') ||
    errorString.includes('response could not be found') ||
    errorMessage.includes('response could not be found')
  );
};

export const isTimeoutError = (error) => {
  if (!error) return false;
  const errorString = error.toString().toLowerCase();
  const errorMessage = (error.message || '').toLowerCase();
  return (
    errorString.includes('request timed out after') ||
    errorMessage.includes('request timed out after')
  );
};

/**
 * Handles errors with special logic for local network signature verification issues
 * and non-local invalid read state errors.
 *
 * This handler should be used in catch blocks where Plug wallet errors occur.
 * On local networks with Plug wallet, signature verification errors are known issues
 * that can be safely ignored. On non-local networks, "Invalid read state request" errors
 * can be safely ignored.
 *
 * @param {Error} error - The error to handle
 * @throws {Error} - Re-throws the error if it's not a known ignorable error
 *
 * @example
 * try {
 *   await ledgerActor.icrc1_transfer(transferArg);
 * } catch (error) {
 *   handlePlugError(error);
 * }
 */
export const handlePlugError = (error) => {
  console.log('handlePlugError', error);
  const currentNetwork = getCurrentNetwork();
  const isLocal = currentNetwork === ICP_NETWORKS.LOCAL;

  // Only ignore signature verification errors on local network
  if (isLocal && (isSignatureVerificationError(error) || isFetchRootKeyError(error))) {
    console.log('Ignoring local plug error:', error.message || error);
    return; // Silently ignore
  }

  // Ignore invalid read state errors on non-local networks
  if (!isLocal && isInvalidReadStateError(error)) {
    console.log('Ignoring invalid read state error on non-local network:', error.message || error);
    return; // Silently ignore
  }

  // Ignore timeout errors on non-local networks
  if (!isLocal && isTimeoutError(error)) {
    console.log('Ignoring timeout error on non-local network:', error.message || error);
    return; // Silently ignore
  }

  // For all other cases, throw the error
  throw error;
};
