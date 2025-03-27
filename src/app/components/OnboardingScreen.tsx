// Modified from original Apache 2.0 licensed code: adjusted for WayStation look and feel

"use client";

import React, { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { cn } from '@/app/lib/utils';
import InstallMcpUI from './InstallMcp';
import { Dialog } from './ui/dialog';
import { Button } from './ui/button';
import { DragRegion } from './ui/drag-region';
import * as DialogPrimitive from '@radix-ui/react-dialog';
import { TextAnimate } from './magicui/text-animate';
import { BlurFade } from './magicui/blur-fade';
import AuthButton from './AuthButton';
import { markOnboardingCompleted } from '@/app/lib/utils/onboarding';

import { openPath } from '@tauri-apps/plugin-opener';

interface OnboardingScreenProps {
  isOpen: boolean;
  onComplete: () => void;
}

// Custom DialogContent without close button
const DialogContentWithoutCloseButton = React.forwardRef<
  React.ElementRef<typeof DialogPrimitive.Content>,
  React.ComponentPropsWithoutRef<typeof DialogPrimitive.Content>
>(({ className, children, ...props }, ref) => (
  <DialogPrimitive.Portal>
    <DialogPrimitive.Overlay className="fixed inset-0 z-50 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0" />
    <DialogPrimitive.Content
      ref={ref}
      className={cn(
        "fixed left-[50%] top-[50%] z-50 grid w-full max-w-lg translate-x-[-50%] translate-y-[-50%] gap-4 border p-6 shadow-lg duration-200 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[state=closed]:slide-out-to-left-1/2 data-[state=closed]:slide-out-to-top-[48%] data-[state=open]:slide-in-from-left-1/2 data-[state=open]:slide-in-from-top-[48%] sm:rounded-lg",
        className
      )}
      {...props}>
    {children}
      {/* Close button removed */}
    </DialogPrimitive.Content>
  </DialogPrimitive.Portal>
));
DialogContentWithoutCloseButton.displayName = "DialogContentWithoutCloseButton";

export function OnboardingScreen({
  isOpen,
  onComplete,
}: OnboardingScreenProps) {
  const [currentStep, setCurrentStep] = useState(0);
  const [isClaudeInstalled, setIsClaudeInstalled] = useState(false);
  const claudeOpened = useRef(false);

  const steps = [
    {
      title: "Welcome to WayStation",
      description: "Start by dragging the WayStation app to Claude",
    },
    {
      title: "Fantastic!",
      description: isClaudeInstalled
        ? `Next, open Claude and write "Hello WayStation"`
        : "Next, download and install Claude",
    },
    {
      title: "Almost there!",
      description: "Sign up or log in to connect. It's free!",
    },
  ];

  useEffect(() => {
    // Check if Claude is installed when component mounts or when step 1 is reached
    if (currentStep === 1) {
      invoke<boolean>("check_claude_installed")
        .then((installed) => {
          setIsClaudeInstalled(installed);
          console.log("Claude installed:", installed);
        })
        .catch((error) => {
          console.error("Failed to check Claude installation:", error);
        });
    }
  }, [currentStep]);

  useEffect(() => {
    if (currentStep !== 1) return;

    const checkOnboardingStatus = async () => {
      if (!claudeOpened.current) return;

      try {
        const isCompleted = await invoke<boolean>("check_onboarding_completed");
        console.log("Onboarding check result:", isCompleted);

        if (isCompleted) {
          setCurrentStep(2);
          markOnboardingCompleted();
          claudeOpened.current = false;
        }
      } catch (error) {
        console.error("Failed to check onboarding status:", error);
      }
    };

    // Set up polling to check status every 2 seconds
    const intervalId = setInterval(checkOnboardingStatus, 2000);

    // Set up window focus handler
    const handleWindowFocus = () => {
      // When window regains focus, check Claude installation status again
      invoke<boolean>("check_claude_installed")
        .then((installed) => {
          setIsClaudeInstalled(installed);
          console.log("Claude installed (focus check):", installed);
        })
        .catch((error) => {
          console.error("Failed to check Claude installation:", error);
        });

      checkOnboardingStatus();
    };

    window.addEventListener("focus", handleWindowFocus);

    return () => {
      window.removeEventListener("focus", handleWindowFocus);
      clearInterval(intervalId);
    };
  }, [currentStep]);

  const onDropSuccess = () => {
    setCurrentStep(currentStep + 1);

    invoke("install_waystation_mcp")
      .then(() => {
        console.log("Successfully installed waystation-mcp");
      })
      .catch((error) => {
        console.error("Failed to install waystation-mcp:", error);
      });
  };

  const handleOpenClaude = () => {
    claudeOpened.current = true; // Set flag indicating Claude was opened

    invoke("restart_claude_app")
      .then(() => {
        console.log("Successfully opened Claude");
      })
      .catch((error) => {
        console.error("Failed to open Claude:", error);
        claudeOpened.current = false; // Reset flag if there was an error
      });
  };

  const handleDownloadClaude = () => {
    openPath("https://claude.ai/download")
    .then(() => {
      console.log("Opened Claude download page");
    })
    .catch((error) => {
      console.error("Failed to open download page:", error);
    });
  };

  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && onComplete()}>
      <DialogContentWithoutCloseButton className="w-screen  max-w-screen h-screen">
        <div className="flex flex-col justify-between h-full w-full py-8 ">
          <DragRegion className="absolute z-overlay top-0 left-0 right-0" />
          <div>
            <div className="relative top-[80px] mx-auto flex w-full items-center justify-center">
              <div>
                <BlurFade direction="down" delay={0.5}>
                  <img
                    className="relative"
                    src="/images/tools/monday.svg"
                    alt="Monday"
                    width={30}
                    height={30}
                  />
                </BlurFade>
                <BlurFade direction="right" delay={1}>
                  <img
                    className="relative right-[18px] bottom-[8px]"
                    src="/images/tools/slack.svg"
                    alt="Slack"
                    width={30}
                    height={30}
                  />
                </BlurFade>
                <BlurFade direction="left" delay={1.5}>
                  <img
                    className="relative bottom-[30px] left-[17px]"
                    src="/images/tools/gdrive.svg"
                    alt="Google Drive"
                    width={26}
                    height={26}
                  />
                </BlurFade>
              </div>
            </div>
            <div className="mt-20">
              <p className="text-[44px] text-center font-bold -tracking-[1px] leading-none">
                <TextAnimate
                  delay={currentStep === 0 ? 2 : 0}
                  animation="blurInUp"
                  by="character">
                  {steps[currentStep].title}
                </TextAnimate>
              </p>
              <p className="text-sm text-center text-[#535d75] mt-2 ">
                <TextAnimate
                  delay={currentStep === 0 ? 2.5 : 0.5}
                  animation="blurInUp"
                  by="character">
                  {steps[currentStep].description}
                </TextAnimate>
              </p>
            </div>
            <div className="mt-10 items-center justify-center flex">
              {currentStep === 0 && (
                <InstallMcpUI onDragSuccess={onDropSuccess} />
              )}
              {currentStep === 1 && (
                <BlurFade delay={1.5}>
                  <div className="flex justify-center">
                    <Button
                      onClick={
                        isClaudeInstalled
                          ? handleOpenClaude
                          : handleDownloadClaude
                      }
                      variant="secondary"
                      className="w-full aurora-btn">
                      {isClaudeInstalled ? "Open Claude" : "Download Claude"}
                    </Button>
                  </div>
                </BlurFade>
              )}
              {currentStep === 2 && (
                <BlurFade delay={1.5}>
                  <div className="flex justify-center">
                    <AuthButton className="aurora-btn px-4 py-2 text-sm font-bold rounded hover:scale-105 transition-transform duration-300 w-auto text-center"/>
                  </div>
                </BlurFade>
              )}
            </div>
          </div>
          <div className="flex justify-between">
            <div className="flex w-full justify-center relative top-[120px]">
              <BlurFade delay={4}>
                <div className="flex justify-center gap-2 mb-4 mt-5">
                  {steps.map((_, index) => (
                    <div
                      key={index}
                      className={cn(
                        "w-2 h-2 rounded-full transition-all",
                        currentStep === index
                          ? "bg-sand-700 w-5"
                          : "bg-sand-200"
                      )}
                    />
                  ))}
                </div>
              </BlurFade>
            </div>
          </div>
        </div>
      </DialogContentWithoutCloseButton>
    </Dialog>
  );
}
