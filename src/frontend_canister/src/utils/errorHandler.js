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

/**
 * Handles errors with special logic for local network signature verification issues
 *
 * This handler should be used in catch blocks where Plug wallet errors occur.
 * On local networks with Plug wallet, signature verification errors are known issues
 * that can be safely ignored. On mainnet or other networks, these errors should be thrown.
 *
 * @param {Error} error - The error to handle
 * @throws {Error} - Re-throws the error if it's not a signature verification error on local network
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

  // For all other cases, throw the error
  throw error;
};
