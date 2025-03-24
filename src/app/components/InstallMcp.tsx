// Modified from original Apache 2.0 licensed code: adjusted for WayStation look and feel


"use client";

import { useEffect, useRef, useState } from 'react';
import { BlurFade } from './magicui/blur-fade';

interface InstallMcpUIProps {
  onDragSuccess?: () => void;
}

export default function InstallMcpUI({ onDragSuccess }: InstallMcpUIProps) {
  const [isDragging, setIsDragging] = useState(false);
  const [isInstalled, setIsInstalled] = useState(false);
  const [isOverTarget, setIsOverTarget] = useState(false);
  const sourceRef = useRef<HTMLDivElement>(null);
  const targetRef = useRef<HTMLDivElement>(null);
  
  // Use a ref to track the clone element without directly assigning to .current
  const cloneElementRef = useRef<HTMLDivElement | null>(null);
  
  // Use a ref to track if we're currently over the target to avoid state timing issues
  const isOverTargetRef = useRef(false);
  
  // Set up the drag clone when dragging starts
  useEffect(() => {
    if (isDragging && sourceRef.current) {
      const source = sourceRef.current;
      const sourceRect = source.getBoundingClientRect();
      
      // Create clone element
      const clone = document.createElement('div');
      clone.className = 'fixed bg-sand-50 w-16 h-16 flex rounded-xl shadow-md items-center justify-center z-50';
      clone.style.position = 'fixed';
      clone.style.width = `${sourceRect.width}px`;
      clone.style.height = `${sourceRect.height}px`;
      clone.style.top = '0';
      clone.style.left = '0';
      clone.style.transform = `translate(${sourceRect.left}px, ${sourceRect.top}px)`;
      clone.style.opacity = '0.9';
      clone.style.pointerEvents = 'none';
      clone.style.zIndex = '9999';
      clone.style.cursor = 'grabbing';
      
      // Add logo image to clone
      const img = document.createElement('img');
      img.src = '/logo.svg';
      img.alt = 'MCP';
      img.style.width = '100%';
      img.style.height = '100%';
      img.style.objectFit = 'contain';
      clone.appendChild(img);
      
      // Add to document
      document.body.appendChild(clone);
      
      // Store reference to the clone
      cloneElementRef.current = clone;
    }
    
    return () => {
      // Clean up clone when dragging stops
      if (cloneElementRef.current) {
        document.body.removeChild(cloneElementRef.current);
        cloneElementRef.current = null;
      }
    };
  }, [isDragging]);
  
  // Handle success with delay
  const handleSuccess = () => {
    if (onDragSuccess) {
      // Delay the callback by 1 second to give users time to see the success state
      setTimeout(() => {
        onDragSuccess();
      }, 1000);
    }
  };
  
  // Check if two elements are overlapping
  const checkOverlap = (element1: DOMRect, element2: DOMRect): boolean => {
    // Make the target area larger for easier dropping
    const padding = 30;
    const expandedElement2 = {
      left: element2.left - padding,
      right: element2.right + padding,
      top: element2.top - padding,
      bottom: element2.bottom + padding
    };
    
    // Check if the center of element1 is within the expanded bounds of element2
    const centerX = element1.left + element1.width / 2;
    const centerY = element1.top + element1.height / 2;
    
    return (
      centerX >= expandedElement2.left &&
      centerX <= expandedElement2.right &&
      centerY >= expandedElement2.top &&
      centerY <= expandedElement2.bottom
    );
  };
  
  // Handle drag start
  const handleMouseDown = (e: React.MouseEvent) => {
    if (isInstalled) return;
    
    e.preventDefault();
    
    const sourceRect = sourceRef.current?.getBoundingClientRect();
    if (!sourceRect) return;
    
    // Calculate offset of mouse within the source element
    const offsetX = e.clientX - sourceRect.left;
    const offsetY = e.clientY - sourceRect.top;
    
    // Start dragging
    setIsDragging(true);
    
    // Add cursor style to body
    document.body.style.cursor = 'grabbing';
    
    // Handle mouse move
    const handleMouseMove = (e: MouseEvent) => {
      e.preventDefault();
      
      if (!cloneElementRef.current) return;
      
      // Update clone position
      const x = e.clientX - offsetX;
      const y = e.clientY - offsetY;
      cloneElementRef.current.style.transform = `translate(${x}px, ${y}px)`;
      
      // Check if over target
      if (targetRef.current) {
        const targetRect = targetRef.current.getBoundingClientRect();
        const cloneRect = cloneElementRef.current.getBoundingClientRect();
        
        // Check for overlap
        const isOver = checkOverlap(cloneRect, targetRect);
        
        // Update both the state and the ref
        setIsOverTarget(isOver);
        isOverTargetRef.current = isOver;
      }
    };
    
    // Handle mouse up
    const handleMouseUp = () => {
      // Reset cursor
      document.body.style.cursor = '';
      
      // Use the ref value for immediate access
      const wasOverTarget = isOverTargetRef.current;
      
      // Handle successful drop
      if (wasOverTarget) {
        console.log('Drop successful!');
        setIsInstalled(true);
        
        // Call the success callback with delay
        handleSuccess();
      }
      
      // End dragging
      setIsDragging(false);
      setIsOverTarget(false);
      isOverTargetRef.current = false;
      
      // Remove event listeners
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
    
    // Add event listeners
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  };
  
  return (
    <div className="flex gap-10 items-center">
      <BlurFade delay={3.6}>
        <div 
          ref={sourceRef}
          className={`relative bg-white w-16 h-16 flex rounded-xl shadow-md items-center justify-center ${isInstalled ? 'opacity-0' : isDragging ? 'cursor-grabbin scale-95 opacity-50' : 'cursor-grab'}`}
          onMouseDown={!isInstalled ? handleMouseDown : undefined}
        >
          <img src="/logo.svg" alt="MCP" />
        </div>
      </BlurFade>
      
      <BlurFade delay={3.7}>
        <img src="/images/arrow.svg" alt="MCP" />
      </BlurFade>
      
      <BlurFade delay={3.8}>
        <div 
          ref={targetRef}
          className={`bg-white w-16 h-16 flex rounded-xl shadow-md items-center justify-center transition-all duration-200 ${isOverTarget ? 'bg-sand-200 scale-110' : ''} ${isInstalled ? 'ring-2 ring-sand-400' : ''}`}
        >
          <img className="w-10 h-10" src="/claude.svg" alt="Claude" />
        </div>
      </BlurFade>
    </div>
  );
}

