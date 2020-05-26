<?xml version="1.0"?>

<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <xsl:output method="text" />
  <xsl:strip-space elements="*"/>
  <xsl:preserve-space elements="s"/>

  <xsl:template match="* | @*">
    <xsl:copy>
      <xsl:apply-templates select="* | @*" />
    </xsl:copy>
  </xsl:template>

  <xsl:template match="/*">
    <xsl:apply-templates />
  </xsl:template>

  <xsl:template match="art">
    <xsl:text># newdoc id = taz-</xsl:text>
    <xsl:value-of select="kopf/@dat" />
    <xsl:text>-</xsl:text>
    <xsl:value-of select="kopf/@nr" />
    <xsl:text>&#xa;</xsl:text>
    <xsl:apply-templates select="kopf/@ti" />
    <xsl:apply-templates select="kopf/@ti2" />
    <xsl:apply-templates select="kopf/@au" />

    <xsl:for-each select=".//s">
      <xsl:apply-templates select=".">
	<xsl:with-param name="sent_id" select="position()" />
      </xsl:apply-templates>
    </xsl:for-each>
  </xsl:template>

  <xsl:template match="@ti">
    <xsl:text># title = </xsl:text>
    <xsl:value-of select="." />
    <xsl:text>&#xa;</xsl:text>
  </xsl:template>

  <xsl:template match="@ti2">
    <xsl:text># subtitle = </xsl:text>
    <xsl:value-of select="." />
    <xsl:text>&#xa;</xsl:text>
  </xsl:template>

  <xsl:template match="@au">
    <xsl:text># author = </xsl:text>
    <xsl:value-of select="." />
    <xsl:text>&#xa;</xsl:text>
  </xsl:template>

  <xsl:template match="s">
    <xsl:param name="sent_id" />
    
    <xsl:text># sent_id = taz-</xsl:text>
    <xsl:value-of select="ancestor::art/kopf/@dat" />
    <xsl:text>-</xsl:text>
    <xsl:value-of select="ancestor::art/kopf/@nr" />
    <xsl:text>-</xsl:text>
    <xsl:value-of select="$sent_id" />
    <xsl:text>&#xa;</xsl:text>
    <xsl:text># text = </xsl:text>
    <xsl:value-of select="normalize-space(.)" />
    <xsl:text>&#xa;</xsl:text>

    <xsl:for-each select=".//t">
      <xsl:value-of select="position()" />
      <xsl:text>&#x9;</xsl:text>
      <xsl:value-of select="@f" />
      <xsl:text>&#xa;</xsl:text>
    </xsl:for-each>

    <xsl:text>&#xa;</xsl:text>
  </xsl:template>
</xsl:stylesheet>
