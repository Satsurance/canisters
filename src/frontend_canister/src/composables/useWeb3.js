import { useWeb3Store } from '../stores/web3Store';

export function useWeb3() {
    const web3Store = useWeb3Store();

    const checkPlugInstalled = () => {
        return typeof window.ic !== 'undefined' && typeof window.ic.plug !== 'undefined';
    };

    const connectWallet = async () => {
        return await web3Store.connectWallet();
    };


    return {
        connectWallet,
        checkPlugInstalled,
        isConnected: () => web3Store.isConnected,
        account: () => web3Store.account,
        chainId: () => web3Store.chainId
    };
}
