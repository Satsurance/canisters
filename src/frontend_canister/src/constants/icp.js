export const ICP_NETWORKS = {
    LOCAL: 'local',
    MAINNET: 'ic',
    TESTNET: 'testnet'
};

export const ICP_MAINNET_BLOCK_EXPLORER = 'https://dashboard.internetcomputer.org';

export const ICP_CONFIG = {
    [ICP_NETWORKS.LOCAL]: {
        name: 'ICP Local',
        host: 'http://127.0.0.1:4943',
        canisters: {
            backend: import.meta.env.VITE_CANISTER_ID_POOL_CANISTER ||'uzt4z-lp777-77774-qaabq-cai',
            ledger: import.meta.env.VITE_CANISTER_ID_ICRC1_LEDGER ||'uxrrr-q7777-77774-qaaaq-cai',
            // claim: import.meta.env.VITE_CANISTER_ID_CLAIM_CANISTER||'umunu-kh777-77774-qaaca-cai',
        }
    },
    [ICP_NETWORKS.MAINNET]: {
        name: 'ICP Mainnet',
        host: 'https://icp0.io',
        canisters: {
            backend: import.meta.env.VITE_CANISTER_ID_POOL_CANISTER,
            ledger: import.meta.env.VITE_CANISTER_ID_ICRC1_LEDGER,
            // claim: import.meta.env.VITE_CANISTER_ID_CLAIM_CANISTER,
        }
    }
};

export const getCurrentNetwork = () => {
    return import.meta.env.VITE_DFX_NETWORK === 'ic' ? ICP_NETWORKS.MAINNET : ICP_NETWORKS.LOCAL;
};

export const getCanisterIds = (network = null) => {
    const currentNetwork = network || getCurrentNetwork();
    return ICP_CONFIG[currentNetwork]?.canisters || ICP_CONFIG[ICP_NETWORKS.LOCAL].canisters;
};
