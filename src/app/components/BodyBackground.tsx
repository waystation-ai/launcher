"use client"
import Image from 'next/image';


export default function BodyBackground() {
  return (
  <div style={{
    position: 'fixed',
    top: 0,
    left: 0,
    width: '100%',
    height: '100%',
    zIndex: -2,
  }}>
    <Image 
      width={800} 
      height={600} 
      src="/images/background.png" 
      alt="Background" 
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        width: '100%',
        height: '100%',
        objectFit: 'cover'
      }}
    />
    <div className="absolute bg-[#00e5a8] rounded-full blur-[400px]" style={{
      width: '37.43%',
      height: '37.43%',
      left: '83.68%',
      top: '0%'
    }} />
    <div className="absolute bg-[#5f9aff] rounded-full blur-[400px]" style={{
      width: '64.86%',
      height: '64.86%',
      left: '0%',
      top: '65.56%'
    }} />
    <div className="absolute opacity-60 bg-white" style={{
      width: '100%',
      height: '100%',

    }}/>

  </div>

  );
}
