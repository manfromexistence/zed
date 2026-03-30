Please look at the components/pixel-circle.tsx file and use this controller to put a pixel circle and its controller in the welcome screen:



```ts

import React, { useState } from 'react';

import { PixelCircle } from './components/PixelCircle';

import { Play, Pause, Settings } from 'lucide-react';

import { motion } from 'motion/react';

const PALETTES = [

  ['#002b00', '#005e00', '#00a800', '#4dff4d', '#ffffff'],

  ['#330000', '#800000', '#e60000', '#ff6600', '#ffcc00', '#ffffff'],

  ['#1a0033', '#4d0099', '#9900cc', '#e600e6', '#ff99ff', '#ffffff'],

  ['#33001a', '#99004d', '#e60073', '#ff4d94', '#ffb3d1', '#ffffff'],

];

export default function App() {

  const [isPlaying, setIsPlaying] = useState(true);

  const [speed, setSpeed] = useState(3);

  const [resolution, setResolution] = useState(12);

  const [circleSize, setCircleSize] = useState(96);

  const [overlap, setOverlap] = useState(32);

  const [noiseAmount, setNoiseAmount] = useState(0.15);

  return (

    <div className="min-h-screen bg-[#0a0a0a] text-neutral-200 flex flex-col items-center justify-center p-8 font-sans selection:bg-white/20">

      

      <div className="absolute top-8 left-8">

        <h1 className="text-2xl font-bold tracking-tighter text-white">Pixel<span className="text-neutral-500">Circles</span></h1>

      </div>

      {/* Canvas Area */}

      <div className="flex-1 flex items-center justify-center w-full min-h-[400px]">

        <motion.div 

          initial={{ opacity: 0, scale: 0.9 }}

          animate={{ opacity: 1, scale: 1 }}

          transition={{ duration: 0.8, ease: "easeOut" }}

          className="flex items-center justify-center" 

        >

          {PALETTES.map((palette, i) => (

            <motion.div 

              key={i} 

              initial={{ x: -20, opacity: 0 }}

              animate={{ x: 0, opacity: 1 }}

              transition={{ duration: 0.5, delay: i * 0.1 + 0.3 }}

              style={{ 

                marginLeft: i === 0 ? 0 : `-${overlap}px`,

                zIndex: PALETTES.length - i 

              }}

              className="relative rounded-full overflow-hidden border-[3px] border-[#0a0a0a] shadow-2xl"

            >

              <PixelCircle 

                palette={palette} 

                speed={speed} 

                resolution={resolution} 

                isPlaying={isPlaying} 

                timeOffset={i * 1000} 

                size={circleSize}

                noiseAmount={noiseAmount}

              />

            </motion.div>

          ))}

        </motion.div>

      </div>

      {/* Controls Area */}

      <motion.div 

        initial={{ opacity: 0, y: 20 }}

        animate={{ opacity: 1, y: 0 }}

        transition={{ duration: 0.6, delay: 0.6 }}

        className="w-full max-w-md bg-neutral-900/80 backdrop-blur-2xl border border-white/5 p-8 rounded-[2rem] shadow-2xl space-y-8"

      >

        <div className="flex items-center justify-between">

          <div>

            <h2 className="text-xl font-semibold text-white flex items-center gap-2 tracking-tight">

              <Settings className="w-5 h-5 text-neutral-400" />

              Parameters

            </h2>

            <p className="text-sm text-neutral-500 mt-1">Adjust the pixelated gradient effect</p>

          </div>

          <button 

            onClick={() => setIsPlaying(!isPlaying)}

            className="w-12 h-12 flex items-center justify-center bg-white text-black hover:bg-neutral-200 rounded-full transition-transform active:scale-95 shadow-lg"

          >

            {isPlaying ? <Pause className="w-5 h-5 fill-current" /> : <Play className="w-5 h-5 fill-current ml-1" />}

          </button>

        </div>

        <div className="space-y-6">

          <ControlSlider 

            label="Animation Speed" 

            value={speed} 

            min={0} 

            max={10} 

            step={0.1} 

            onChange={setSpeed} 

          />

          <ControlSlider 

            label="Pixel Density" 

            value={resolution} 

            min={4} 

            max={48} 

            step={1} 

            onChange={setResolution} 

          />

          <ControlSlider 

            label="Circle Size" 

            value={circleSize} 

            min={40} 

            max={200} 

            step={4} 

            onChange={setCircleSize} 

          />

          <ControlSlider 

            label="Overlap Offset" 

            value={overlap} 

            min={0} 

            max={circleSize} 

            step={1} 

            onChange={setOverlap} 

          />

          <ControlSlider 

            label="Static Noise" 

            value={noiseAmount} 

            min={0} 

            max={0.5} 

            step={0.01} 

            onChange={setNoiseAmount} 

          />

        </div>

      </motion.div>

    </div>

  );

}

function ControlSlider({ label, value, min, max, step, onChange }: { label: string, value: number, min: number, max: number, step: number, onChange: (val: number) => void }) {

  return (

    <div className="space-y-3">

      <div className="flex justify-between text-sm font-medium">

        <label className="text-neutral-300">{label}</label>

        <span className="text-neutral-500 font-mono">{value.toFixed(step % 1 !== 0 ? 1 : 0)}</span>

      </div>

      <input 

        type="range" 

        min={min} 

        max={max} 

        step={step} 

        value={value} 

        onChange={(e) => onChange(parseFloat(e.target.value))}

        className="w-full h-1.5 bg-neutral-800 rounded-full appearance-none cursor-pointer accent-white hover:accent-neutral-300 transition-all"

      />

    </div>

  );

}

```