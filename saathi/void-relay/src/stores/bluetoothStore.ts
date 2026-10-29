import { create } from 'zustand';

export interface BTDevice {
  id: string;
  name: string;
  address: string;
  connected: boolean;
}

interface BluetoothStore {
  devices: BTDevice[];
  isScanning: boolean;
  hfpActiveAndroid: boolean;
  hfpActiveIos: boolean;
  setDevices: (devices: BTDevice[]) => void;
  setScanning: (scanning: boolean) => void;
  setHfpActiveAndroid: (active: boolean) => void;
  setHfpActiveIos: (active: boolean) => void;
}

export const useBluetoothStore = create<BluetoothStore>((set) => ({
  devices: [],
  isScanning: false,
  hfpActiveAndroid: false,
  hfpActiveIos: false,
  setDevices: (devices) => set({ devices }),
  setScanning: (isScanning) => set({ isScanning }),
  setHfpActiveAndroid: (hfpActiveAndroid) => set({ hfpActiveAndroid }),
  setHfpActiveIos: (hfpActiveIos) => set({ hfpActiveIos }),
}));
