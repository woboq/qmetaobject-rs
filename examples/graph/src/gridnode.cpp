/****************************************************************************
**
** Copyright (C) 2017 The Qt Company Ltd.
** Contact: https://www.qt.io/licensing/
**
** This file is part of the examples of the Qt Toolkit.
**
** $QT_BEGIN_LICENSE:BSD$
** Commercial License Usage
** Licensees holding valid commercial Qt licenses may use this file in
** accordance with the commercial license agreement provided with the
** Software or, alternatively, in accordance with the terms contained in
** a written agreement between you and The Qt Company. For licensing terms
** and conditions see https://www.qt.io/terms-conditions. For further
** information use the contact form at https://www.qt.io/contact-us.
**
** BSD License Usage
** Alternatively, you may use this file under the terms of the BSD license
** as follows:
**
** "Redistribution and use in source and binary forms, with or without
** modification, are permitted provided that the following conditions are
** met:
**   * Redistributions of source code must retain the above copyright
**     notice, this list of conditions and the following disclaimer.
**   * Redistributions in binary form must reproduce the above copyright
**     notice, this list of conditions and the following disclaimer in
**     the documentation and/or other materials provided with the
**     distribution.
**   * Neither the name of The Qt Company Ltd nor the names of its
**     contributors may be used to endorse or promote products derived
**     from this software without specific prior written permission.
**
**
** THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
** "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
** LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
** A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
** OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
** SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
** LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
** DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
** THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
** (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
** OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE."
**
** $QT_END_LICENSE$
**
****************************************************************************/

#include "gridnode.h"

#include "qmath.h"

#define GRID_SIZE 32

GridNode::GridNode()
    : m_geometry(QSGGeometry::defaultAttributes_Point2D(), 0)
{
    setGeometry(&m_geometry);
    m_geometry.setDrawingMode(GL_LINES);

    setMaterial(&m_material);
    m_material.setColor(Qt::gray);
}

/*
 * The function hardcodes a fixed set of grid lines and scales
 * those to the bounding rect.
 */
void GridNode::setRect(const QRectF &rect)
{
    int vCount = int((rect.width() - 1) / GRID_SIZE);
    int hCount = int((rect.height() - 1) / GRID_SIZE);

    int lineCount = vCount + hCount;

    QSGGeometry *g = geometry();

    g->allocate(lineCount * 2);

    float x = rect.x();
    float y = rect.y();
    float w = rect.width();
    float h = rect.height();

    QSGGeometry::Point2D *v = g->vertexDataAsPoint2D();

    // Then write the vertical lines
    for (int i=0; i<vCount; ++i)  {
        float dx = (i + 1) * GRID_SIZE;
        v[i*2].set(dx, y);
        v[i*2+1].set(dx, y + h);
    }
    v += vCount * 2;
    // Then write the horizontal lines
    for (int i=0; i<hCount; ++i)  {
        float dy = (i + 1) * GRID_SIZE;
        v[i*2].set(x, dy);
        v[i*2+1].set(x + w, dy);
    }

    // Tell the scenegraph we updated the geometry..
    markDirty(QSGNode::DirtyGeometry);
}
