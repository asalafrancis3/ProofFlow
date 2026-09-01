export interface ContractConfig {
  contractId: string;
  network: 'TESTNET' | 'MAINNET' | 'STANDALONE';
  rpcUrl: string;
}

interface AppConfig {
  contract: ContractConfig;
  api: {
    baseUrl: string;
    timeout: number;
  };
  features: {
    offlineMode: boolean;
    analytics: boolean;
    notifications: boolean;
  };
  logging: {
    level: 'debug' | 'info' | 'warn' | 'error';
    enableConsole: boolean;
  };
}

function validateContractConfig(config: ContractConfig): void {
  if (!config.contractId) throw new Error('VITE_CONTRACT_ID is required.');
  if (!config.network) throw new Error('VITE_NETWORK is required.');
  if (!config.rpcUrl) throw new Error('VITE_RPC_URL is required.');
}

function loadConfig(): AppConfig {
  const contractConfig: ContractConfig = {
    contractId: import.meta.env.VITE_CONTRACT_ID || '',
    network: (import.meta.env.VITE_NETWORK || 'TESTNET') as ContractConfig['network'],
    rpcUrl: import.meta.env.VITE_RPC_URL || '',
  };

  validateContractConfig(contractConfig);

  return {
    contract: contractConfig,
    api: {
      baseUrl: import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000',
      timeout: parseInt(import.meta.env.VITE_API_TIMEOUT || '30000', 10),
    },
    features: {
      offlineMode: import.meta.env.VITE_OFFLINE_MODE === 'true',
      analytics: import.meta.env.VITE_ANALYTICS === 'true',
      notifications: import.meta.env.VITE_NOTIFICATIONS !== 'false',
    },
    logging: {
      level: (import.meta.env.VITE_LOG_LEVEL || 'info') as AppConfig['logging']['level'],
      enableConsole: import.meta.env.VITE_LOG_CONSOLE !== 'false',
    },
  };
}

export const config = loadConfig();
