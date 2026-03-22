import * as React from "react";
import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

/**
 * Button variant definitions using CVA (Class Variance Authority).
 *
 * CVA creates a function that returns different Tailwind classes based on props.
 * Think of it as a type-safe way to define component variants.
 *
 * The first argument is the BASE classes (always applied).
 * `variants` defines the possible variations, each with named options.
 * `defaultVariants` sets what's used when no variant prop is provided.
 */
const buttonVariants = cva(
  // Base classes - always applied to every button
  "inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50",
  {
    variants: {
      variant: {
        default: "bg-primary text-primary-foreground hover:bg-primary/90",
        destructive: "bg-destructive text-destructive-foreground hover:bg-destructive/90",
        outline: "border border-input bg-background hover:bg-accent hover:text-accent-foreground",
        secondary: "bg-secondary text-secondary-foreground hover:bg-secondary/80",
        ghost: "hover:bg-accent hover:text-accent-foreground",
        link: "text-primary underline-offset-4 hover:underline",
      },
      size: {
        default: "h-10 px-4 py-2",
        sm: "h-9 rounded-md px-3",
        lg: "h-11 rounded-md px-8",
        icon: "h-10 w-10",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
);

/**
 * Button props extend native HTML button attributes AND the CVA variants.
 *
 * `React.ButtonHTMLAttributes<HTMLButtonElement>` gives us all native props
 * (onClick, disabled, type, etc.) without listing them manually.
 *
 * `VariantProps<typeof buttonVariants>` extracts { variant?: "default" | "destructive" | ..., size?: ... }
 * from our CVA definition. This is fully type-safe — TypeScript will error
 * if you pass an invalid variant.
 */
export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean;
}

/**
 * The Button component.
 *
 * `React.forwardRef` creates a component that can receive a `ref` prop.
 * Refs give parent components direct access to the underlying DOM element.
 * This is important for accessibility, focus management, and animations.
 *
 * Without forwardRef, `<Button ref={myRef} />` would not work.
 */
const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, asChild = false, ...props }, ref) => {
    // If `asChild` is true, render the child element with button styles
    // instead of a <button>. This is Radix's "render delegation" pattern.
    const Comp = asChild ? Slot : "button";
    return (
      <Comp
        className={cn(buttonVariants({ variant, size, className }))}
        ref={ref}
        {...props}
      />
    );
  }
);
Button.displayName = "Button";

export { Button, buttonVariants };
