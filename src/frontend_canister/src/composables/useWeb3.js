import { onMounted, onUnmounted } from 'vue';
import { useWeb3Store } from '../stores/web3Store';
import { getCurrentNetwork, getCanisterIds, ICP_CONFIG } from '../constants/icp';

export function useWeb3() {
    const web3Store = useWeb3Store();
    let connectionCheckInterval = null;

    const checkPlugInstalled = () => {
        return typeof window.ic !== 'undefined' && typeof window.ic.plug !== 'undefined';
    };

    const connectWallet = async () => {
        if (!checkPlugInstalled()) {
            throw new Error('Plug wallet is not installed');
        }
        
        try {
            const currentNetwork = getCurrentNetwork();
            const canisters = getCanisterIds(currentNetwork);
            const networkConfig = ICP_CONFIG[currentNetwork];
            
            const isConnected = await window.ic.plug.requestConnect({
                whitelist: Object.values(canisters),
                host: networkConfig.host
            });

            if (isConnected) {
                const principal = await window.ic.plug.agent.getPrincipal();
                web3Store.connect({
                    account: principal.toString(),
                    chainId: currentNetwork,
                    isConnected: true
                });
                
                startConnectionMonitoring();
            }
            
            return isConnected;
        } catch (error) {
            console.error('Failed to connect Plug wallet:', error);
            throw error;
        }
    };

    const checkPlugConnection = async () => {
        console.log('Checking Plug installation...');
        if (!checkPlugInstalled()) {
            console.log('Plug not installed');
            return false;
        }
        
        try {
            if (!window.ic.plug.agent) {
                console.log('No Plug agent found');
                return false;
            }
            console.log('Getting principal...');
            const principal = await window.ic.plug.agent.getPrincipal();
            console.log('Principal:', principal?.toString());
            const isValid = principal && principal.toString() !== '2vxsx-fae';
            console.log('Connection valid:', isValid);
            return isValid;
        } catch (error) {
            console.log('Error in checkPlugConnection:', error);
            return false;
        }
    };

    // Monitor connection status periodically
    const startConnectionMonitoring = () => {
        if (connectionCheckInterval) {
            clearInterval(connectionCheckInterval);
        }
        
        connectionCheckInterval = setInterval(async () => {
            if (web3Store.isConnected) {
                const isStillConnected = await checkPlugConnection();
                if (!isStillConnected) {
                    web3Store.disconnect();
                    clearInterval(connectionCheckInterval);
                }
            }
        }, 5000);
    };

    onMounted(async () => {
        console.log('Starting auto-connect process...');
        await new Promise(resolve => setTimeout(resolve, 500));
        const wasRestored = web3Store.restoreConnection();
        console.log('Restoration result:', wasRestored);
        
        if (wasRestored && checkPlugInstalled()) {
            try {
                console.log('Checking Plug connection...');
                await new Promise(resolve => setTimeout(resolve, 500));
                const isStillConnected = await checkPlugConnection();
                console.log('Connection check result:', isStillConnected);
                
                if (isStillConnected) {
                    web3Store.plugAgent = window.ic.plug.agent;
                    startConnectionMonitoring();
                    console.log('Auto-connect successful!');
                } else {
                    console.log('Connection check failed, disconnecting...');
                    web3Store.disconnect();
                }
            } catch (error) {
                console.error('Error checking wallet connection:', error);
                web3Store.disconnect();
            }
        }
    });

    onUnmounted(() => {
        if (connectionCheckInterval) {
            clearInterval(connectionCheckInterval);
        }
    });

    return {
        connectWallet,
        checkPlugInstalled,
        isConnected: () => web3Store.isConnected,
        account: () => web3Store.account,
        chainId: () => web3Store.chainId
    };
}
