import React from "react";
import logoSrc from "../../assets/logo.png";

const HandyTextLogo = ({
  width,
  height,
  className,
}: {
  width?: number;
  height?: number;
  className?: string;
}) => {
  return (
    <img
      src={logoSrc}
      width={width}
      height={height}
      className={className}
      alt="HandyToFile"
      style={{ objectFit: "contain" }}
    />
  );
};

export default HandyTextLogo;
